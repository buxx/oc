use derive_more::Constructor;
use glam::Vec2;
use oc_individual::{
    Gesture, Individual, IndividualIndex, Update,
    behavior::{Behavior, Intent, MovePath},
    order::Order,
};
use oc_physics::Force;
use oc_root::{
    geo::{WorldVec2, WorldVec3},
    y::V,
};
use oc_utils::{
    d2::{AlmostEqual, Direction},
    let_some,
};
use oc_world::World;

use crate::{index::Indexes, individual::situation::Situation, runner};

pub mod situation;
pub mod update;

const POSITION_TOLERANCE: f32 = 3.0;

// Below this distance to the current waypoint, the walking force is scaled
// down proportionally instead of applied at full strength. This prevents the
// individual from overshooting POSITION_TOLERANCE every tick and bouncing
// back and forth ("circling") around its target.
const ARRIVAL_RADIUS: f32 = 8.0;

// Force is never scaled below this factor, so approach still makes progress
// instead of asymptotically crawling forever.
const MIN_FORCE_FACTOR: f32 = 0.05;

#[derive(Constructor)]
pub struct Processor<'a> {
    world: &'a World,
    index: &'a Indexes,
    i: IndividualIndex,
}

impl<'a> Processor<'a> {
    pub fn step(self) -> Vec<runner::update::Update> {
        tracing::trace!(name="individual-step", i=?self.i);
        let mut updates = vec![];

        let individual = self.world.individual(self.i);
        if !individual.status.can_step() {
            tracing::trace!(name="individual-step-cant-step", i=?self.i);
            return vec![];
        }

        if let Some(updates) = self.accomplished() {
            tracing::trace!(name = "individual-step-accomplished-updates", i=?self.i, updates=?updates);
            return updates;
        }

        let situation = &self.situation();
        let distribute = self.distribute();
        let intent = self.decide(situation);
        let behavior = self.act(situation, &intent);
        let gesture = self.gesture(situation, &behavior);
        let forces = self.forces(individual, &behavior, &intent);

        tracing::trace!(
            name = "individual-step-with",
            i = ?self.i,
            individual = ?individual,
            distribute = ?distribute,
            intent = ?intent,
            behavior = ?behavior,
            gesture = ?gesture,
            forces = ?forces,
        );

        // Dispatch orders to members if not already own it
        for (member_i, orders) in distribute {
            let member = self.world.individual(member_i);
            if member.orders != orders {
                let update = Update::SetOrders(orders);
                let update = runner::update::Update::UpdateIndividual(member_i, update);
                updates.push(update);
            }
        }

        macro_rules! push_update {
            ($self:expr, $updates:expr, $individual:expr, $( $field:ident => $variant:ident ),+ $(,)?) => {
                $(
                    if $field != $individual.$field {
                        $updates.push(runner::update::Update::UpdateIndividual(
                            $self.i,
                            Update::$variant($field),
                        ));
                    }
                )+
            };
        }

        push_update!(self, updates, individual,
            intent => SetIntent,
            behavior => SetBehavior,
            gesture => SetGesture,
            forces => SetForces,
        );

        tracing::trace!(name = "individual-step-updates", i=?self.i, updates=?updates);
        updates
    }

    /// Determine if current order (or order step) is accomplished
    fn accomplished(&self) -> Option<Vec<runner::update::Update>> {
        let individual = self.world.individual(self.i);
        let Some(order) = individual.orders.first() else {
            tracing::trace!(name = "individual-step-accomplished-no-order", i=?self.i);
            return None;
        };
        let squad_i = self.index.individual_squad(self.i);
        let squad = self.world.squad(squad_i);
        let is_squad_leader = self.i == squad.leader();
        let mut updates = None;
        let direction = individual.gesture.direction();

        // Builds the "accomplished" update batch for given individual, appending squad-level
        // updates when this individual is a squad leader.
        let accomplished_updates =
            |i: IndividualIndex, direction: Direction| -> Vec<runner::update::Update> {
                let i_accomplish = Update::Accomplished;
                let i_accomplish = runner::update::Update::UpdateIndividual(i, i_accomplish);

                let i_idle = Update::SetIntent(Intent::Idle(direction));
                let i_idle = runner::update::Update::UpdateIndividual(i, i_idle);

                let mut updates = vec![i_accomplish, i_idle];

                if is_squad_leader {
                    // FIXME: Must wait all members finished associated order (only relevant for MoveTo,
                    // kept here since Idle currently shares the same behavior).
                    let accomplish = oc_individual::squad::Update::Accomplished;
                    let accomplish = runner::update::Update::UpdateSquad(squad_i, accomplish);

                    let orders = squad.orders.clone().into_iter().skip(1).collect();
                    let orders = oc_individual::squad::Update::SetOrders(orders);
                    let orders = runner::update::Update::UpdateSquad(squad_i, orders);

                    updates.extend(vec![accomplish, orders]);
                }

                updates
            };

        match order {
            Order::Idle => {
                tracing::trace!(name = "individual-step-accomplished-idle-finished", i = ?self.i);
                updates = Some(accomplished_updates(self.i, direction));
            }
            Order::MoveTo(position) | Order::MoveFastTo(position) => {
                if position.almost_equal(individual.position, POSITION_TOLERANCE) {
                    tracing::trace!(name = "individual-step-accomplished-move-to-finished", i = ?self.i);
                    updates = Some(accomplished_updates(self.i, direction));
                }
            }
        };

        if updates.is_some() {
            return updates;
        }

        match &individual.intent {
            Intent::Idle(_) => {}
            Intent::MoveTo(_, move_path) | Intent::MoveFastTo(_, move_path) => {
                let Some(next) = move_path.iter().next() else {
                    tracing::trace!(name = "individual-step-accomplished-intent-move-to-no-next", i=?self.i);
                    return updates;
                };

                if next.almost_equal(individual.position, POSITION_TOLERANCE) {
                    let update = Update::MoveStepAccomplished;
                    let update = runner::update::Update::UpdateIndividual(self.i, update);
                    updates.get_or_insert(vec![]).push(update);
                }
            }
        }

        updates
    }

    /// Build updates to ensure each member of squad receive order according to situation.
    fn distribute(&self) -> Vec<(IndividualIndex, Vec<Order>)> {
        let squad_i = self.index.individual_squad(self.i);
        let squad = self.world.squad(squad_i);
        let order = squad.orders.first();
        let mut distribution = Vec::with_capacity(squad.members.len());
        let is_squad_leader = self.i == squad.leader();

        if !is_squad_leader {
            tracing::trace!(name="individual-step-distribute-not-leader", i=?self.i);
            return distribution;
        }

        if let Some(order) = order {
            // Squad leader own the squad order
            distribution.push((squad.leader(), vec![order.clone()]));
        };

        let leader = self.world.individual(squad.leader());
        let gesture = &leader.gesture;
        let reference = Vec2::new(leader.position.x, leader.position.y);
        let count = squad.actives as usize;
        let direction = gesture.direction();
        let angle = direction.angle(V::Server);

        let positions = squad
            .formation
            .positions(&self.world.w, V::Server, reference, angle, count)
            .into_iter()
            .map(|p| WorldVec2::new(p.x, p.y));
        tracing::trace!(name="individual-step-distribute-formation", i=?self.i, squad_i=?squad_i, reference=?reference, direction=?direction, angle=?angle, positions=?positions);

        for (member, position) in squad.members.iter().zip(positions).skip(1) {
            let individual = self.world.individual(*member);
            if position.almost_equal(individual.position, POSITION_TOLERANCE) {
                tracing::trace!(name="individual-step-distribute-already-on-position", i=?self.i, squad_i=?squad_i, member=?member, position=?position);
                continue;
            }

            // According to order, choose appropriate order to distribute (move if move, move fast if move fast, etc.)
            let orders = match order {
                Some(order) => match order {
                    Order::Idle => vec![Order::MoveTo(position.into())],
                    Order::MoveTo(_) => vec![Order::MoveTo(position.into())],
                    Order::MoveFastTo(_) => vec![Order::MoveFastTo(position.into())],
                },
                None => {
                    vec![Order::MoveTo(position.into())]
                }
            };
            tracing::trace!(name="individual-step-distribute-to", i=?self.i, squad_i=?squad_i, order=?order, member=?member, orders=?orders);
            distribution.push((*member, orders))
        }

        tracing::trace!(name="individual-step-distribution", i=?self.i, squad_i=?squad_i, order=?order, distribution=?distribution);
        distribution
    }

    /// Build object which reflect individual situation against environment
    fn situation(&self) -> Situation {
        let enemy_visible = self
            .world
            .visibilities
            .for_(self.i)
            .iter()
            .any(|v| v.visible);

        Situation { enemy_visible }
    }

    /// Decide the individual's intent for this tick.
    fn decide(&self, _situation: &Situation) -> Intent {
        let individual = self.world.individual(self.i);
        let order = individual.orders.first();

        let intent = match individual.can_follow_order() {
            // TODO: things which can prohibe follow order
            true => match order {
                None | Some(Order::Idle) => self.resolve_idle_intent(individual),
                Some(Order::MoveTo(position)) => {
                    let current = individual.intent.path();
                    self.resolve_move_intent(individual, *position, current, Intent::MoveTo)
                }

                Some(Order::MoveFastTo(position)) => {
                    let current = individual.intent.path();
                    self.resolve_move_intent(individual, *position, current, Intent::MoveFastTo)
                }
            },
            false => individual.intent.clone(),
        };

        tracing::trace!(name="individual-step-decide", i=?self.i, order=?order, intent=?intent);
        intent
    }

    // TODO: idle -> Behavior::TakingCover when under fire
    fn act(&self, _situation: &Situation, intent: &Intent) -> Behavior {
        let individual = self.world.individual(self.i);

        match intent {
            Intent::Idle(direction) => Behavior::Idle(direction.clone()),
            Intent::MoveTo(_, path) => match self.direction_to_next(individual, path, "move") {
                Some(direction) => Behavior::Walk(direction),
                None => Behavior::Idle(Direction::NORTH),
            },

            Intent::MoveFastTo(_, path) => {
                match self.direction_to_next(individual, path, "move-fast") {
                    Some(direction) => Behavior::Run(direction),
                    None => Behavior::Idle(Direction::NORTH),
                }
            }
        }
    }

    fn gesture(&self, situation: &Situation, behavior: &Behavior) -> Gesture {
        // FIXME BS NOW: gesture Prone when enemy visible or underfire
        match behavior {
            Behavior::Idle(direction) => match situation.imply_hide() {
                true => Gesture::Prone(*direction),
                false => Gesture::Idle(*direction),
            },
            Behavior::Walk(direction) => Gesture::Walking(*direction),
            Behavior::Run(direction) => Gesture::Running(*direction),
        }
    }

    /// Compute forces for the current behavior.
    fn forces(&self, individual: &Individual, behavior: &Behavior, intent: &Intent) -> Vec<Force> {
        match behavior {
            Behavior::Idle(_) => vec![],
            Behavior::Walk(direction) | Behavior::Run(direction) => {
                let nominal_speed = behavior.nominal_speed();
                let move_disability_factor = self.move_disability_factor(individual, behavior);
                let arrival_factor = self.arrival_factor(intent);
                let direction = WorldVec3::new(direction.x, direction.y, 0.);
                vec![Force::Translation(
                    direction.into(),
                    nominal_speed * move_disability_factor * arrival_factor,
                )]
            }
        }
    }

    fn move_disability_factor(&self, _individual: &Individual, _behavior: &Behavior) -> f32 {
        // TODO: here use fatigue, injures, etc
        1.0
    }

    /// Returns a value in [MIN_FORCE_FACTOR, 1.0] based on distance to the
    /// current waypoint: 1.0 far away, tapering down to MIN_FORCE_FACTOR
    /// within ARRIVAL_RADIUS. We compute it to ensure arrival near the targeted
    /// pixel and avoid turning around target.
    fn arrival_factor(&self, intent: &Intent) -> f32 {
        let path = match intent {
            Intent::Idle(_) => return 1.0, // fallback (should not happens)
            Intent::MoveTo(_, path) => path,
            Intent::MoveFastTo(_, path) => path,
        };
        let_some!(next = path.iter().next(), return 1.0); // 1.0 is fallback (should not happens)

        let individual = self.world.individual(self.i);
        let from = WorldVec2::from(individual.position);
        let distance = *next - from;
        let distance = Vec2::new(distance.x, distance.y).length();

        // No chance to miss the target, return full speed
        if distance >= ARRIVAL_RADIUS {
            1.0
        } else {
            // FIXME: 0.5 is a hack to reduce "missing target".
            // This arrival factor should probably consider the tick delay between each compute
            ((distance / ARRIVAL_RADIUS) * 0.5).max(MIN_FORCE_FACTOR)
        }
    }

    fn resolve_idle_intent(&self, individual: &Individual) -> Intent {
        let direction = individual.gesture.direction();
        Intent::Idle(direction)
    }

    fn resolve_move_intent(
        &self,
        individual: &Individual,
        position: WorldVec2,
        current_path: Option<(WorldVec2, &MovePath)>,
        intent: impl FnOnce(WorldVec2, MovePath) -> Intent,
    ) -> Intent {
        let direction = individual.gesture.direction();

        // Reuse the existing path if we're already moving toward this exact
        // target — don't recompute it every tick.
        if let Some((current_target, current_path)) = current_path {
            if current_target == position && current_path.iter().next().is_some() {
                return individual.intent.clone();
            }
        }

        let from = (individual.position.x, individual.position.y);
        let to = (position.x, position.y);

        match self.world.navmesh.path(from, to) {
            Some(path) => intent(position.clone(), MovePath::from(path)),
            None => {
                tracing::debug!("no path from {:?} to {:?}, falling back to Idle", from, to);
                Intent::Idle(direction)
            }
        }
    }

    fn direction_to_next(
        &self,
        individual: &Individual,
        path: &MovePath,
        trace_name: &'static str,
    ) -> Option<Direction> {
        let Some(next) = path.iter().next() else {
            tracing::trace!(name = format!("individual-step-act-{trace_name}-no-next"), i=?self.i);
            return None;
        };

        let from = WorldVec2::from(individual.position);
        let direction = (*next - from).normalize_or_zero();
        Some(Direction::from(direction))
    }
}

#[cfg(test)]
mod tests {
    use oc_individual::{
        Gesture, IndividualIndex,
        behavior::{Behavior, Intent},
        order::Order,
        squad::SquadIndex,
    };
    use oc_root::{
        WorldConfig,
        geo::{WorldVec2, WorldVec3},
        physics::Meters,
    };
    use oc_utils::d2::Direction;
    use oc_world::World;

    use crate::{index::Indexes, individual::Processor, runner::update::Update};
    use tests::{individual::TestIndividual, squad::TestSquad, world::TestWorld};

    // Test orders distribution when squad own move to order
    #[test]
    fn test_distribute_move() {
        // Given
        let w = WorldConfig::new(100, 100, Meters(0.1))
            .formation_tiles_between_positions(2)
            .geo_pixels_per_tile(5);
        // test parameters (assume individual are all Idle in EST direction)
        let individual_1_position = WorldVec3::new(100., 100., 0.);
        let individual_2_position = WorldVec3::new(90., 110., 0.);
        let squad_position = WorldVec2::new(individual_1_position.x, individual_1_position.y);
        let move_to_position = WorldVec2::new(150., 100.);
        let move_to_order = Order::MoveTo(move_to_position);
        // expected
        let expected_individual_1_move_to_position = WorldVec2::new(150., 100.);
        let expected_individual_2_move_to_position = WorldVec2::new(100., 110.);

        let world = two_individuals_world(
            &w,
            individual_1_position,
            individual_2_position,
            squad_position,
            vec![move_to_order],
        );
        let index = Indexes::new(&world, &w);
        let processor = Processor::new(&world, &index, 0.into());

        // When
        let updates = processor.step();

        // Then
        assert_eq!(
            updates,
            vec![
                Update::UpdateIndividual(
                    IndividualIndex(0),
                    oc_individual::Update::SetOrders(vec![Order::MoveTo(
                        expected_individual_1_move_to_position
                    )])
                ),
                Update::UpdateIndividual(
                    IndividualIndex(1),
                    oc_individual::Update::SetOrders(vec![Order::MoveTo(
                        expected_individual_2_move_to_position
                    )])
                )
            ]
        );
    }

    // Test orders distribution when squad have no order and squad member not at correct place
    #[test]
    fn test_distribute_idle() {
        // Given
        let w = WorldConfig::new(100, 100, Meters(0.1))
            .formation_tiles_between_positions(2)
            .geo_pixels_per_tile(5);
        // test parameters (assume individual are all Idle in EST direction)
        let individual_1_position = WorldVec3::new(100., 100., 0.);
        let individual_2_position = WorldVec3::new(90., 110., 0.);
        let squad_position = WorldVec2::new(individual_1_position.x, individual_1_position.y);
        // expected
        let expected_individual_2_move_to_position = WorldVec2::new(100., 110.);

        let world = two_individuals_world(
            &w,
            individual_1_position,
            individual_2_position,
            squad_position,
            vec![],
        );
        let index = Indexes::new(&world, &w);
        let processor = Processor::new(&world, &index, 0.into());

        // When
        let updates = processor.step();

        // Then
        assert_eq!(
            updates,
            vec![Update::UpdateIndividual(
                IndividualIndex(1),
                oc_individual::Update::SetOrders(vec![Order::MoveTo(
                    expected_individual_2_move_to_position
                )])
            )]
        );
    }

    // Test update when individual have no order
    #[test]
    fn test_idle() {
        // Given
        let w = WorldConfig::new(100, 100, Meters(0.1))
            .formation_tiles_between_positions(2)
            .geo_pixels_per_tile(5);
        // test parameters (assume individual are all Idle in EST direction)
        let position = WorldVec3::new(100., 100., 0.);

        let world = one_individual_world(&w, position, vec![]);
        let index = Indexes::new(&world, &w);
        let processor = Processor::new(&world, &index, 0.into());

        // When
        let updates = processor.step();

        // Then
        assert_eq!(updates, vec![]);
    }

    // Test update when individual have idle order in other direction than current
    #[test]
    fn test_idle_order() {
        // Given
        let w = WorldConfig::new(100, 100, Meters(0.1))
            .formation_tiles_between_positions(2)
            .geo_pixels_per_tile(5);
        // test parameters (assume individual are all Idle in EST direction)
        let position = WorldVec3::new(100., 100., 0.);

        let mut world = one_individual_world(&w, position, vec![Order::Idle]);
        let index = Indexes::new(&world, &w);

        // When-Then
        {
            let processor = Processor::new(&world, &index, 0.into());
            let updates = processor.step();
            assert_eq!(
                updates,
                vec![Update::UpdateIndividual(
                    IndividualIndex(0),
                    oc_individual::Update::SetOrders(vec![Order::Idle])
                )]
            );
        }

        // When-Then
        world.individuals[0].orders = vec![Order::Idle];
        let processor = Processor::new(&world, &index, 0.into());
        let updates = processor.step();
        assert_eq!(
            updates,
            vec![
                Update::UpdateIndividual(IndividualIndex(0), oc_individual::Update::Accomplished),
                Update::UpdateIndividual(
                    IndividualIndex(0),
                    oc_individual::Update::SetIntent(Intent::Idle(Direction::EST))
                ),
                Update::UpdateSquad(SquadIndex(0), oc_individual::squad::Update::Accomplished),
                Update::UpdateSquad(
                    SquadIndex(0),
                    oc_individual::squad::Update::SetOrders(vec![])
                ),
            ]
        );
    }

    // Refactored function which generate a world with one squad composed of two members.
    // Both individuals Idle in EST direction.
    fn two_individuals_world(
        w: &WorldConfig,
        individual_1_position: WorldVec3,
        individual_2_position: WorldVec3,
        squad_position: WorldVec2,
        squad_orders: Vec<Order>,
    ) -> World {
        let individual1 = TestIndividual::builder();
        let individual1 = individual1.position(individual_1_position);
        let individual1 = individual1
            .gesture(Gesture::Idle(Direction::EST)) // Gesture & Behavior & Intent are important
            .behavior(Behavior::Idle(Direction::EST)) // to conditionate the .step() response
            .intent(Intent::Idle(Direction::EST));
        let individual1 = individual1.build().make(w);
        let individual2 = TestIndividual::builder();
        let individual2 = individual2.position(individual_2_position);
        let individual2 = individual2
            .gesture(Gesture::Idle(Direction::EST)) // Gesture & Behavior & Intent are important
            .behavior(Behavior::Idle(Direction::EST)) // to conditionate the .step() response
            .intent(Intent::Idle(Direction::EST));
        let individual2 = individual2.build().make(w);

        let squad = TestSquad::builder();
        let squad = squad.position(squad_position);
        let squad = squad.members(vec![0.into(), 1.into()]);
        let squad = squad.orders(squad_orders);
        let squad = squad.build().make();

        let world = TestWorld::builder();
        let world = world.individuals(vec![individual1, individual2]);
        let world = world.squads(vec![squad]);
        let world = world.build().make(w);

        world
    }

    // Refactored function which generate a world with one squad composed of one member.
    // Individuals Idle in EST direction.
    fn one_individual_world(w: &WorldConfig, position: WorldVec3, orders: Vec<Order>) -> World {
        let individual = TestIndividual::builder();
        let individual = individual.position(position);
        let individual = individual
            .gesture(Gesture::Idle(Direction::EST)) // Gesture & Behavior & Intent are important
            .behavior(Behavior::Idle(Direction::EST)) // to conditionate the .step() response
            .intent(Intent::Idle(Direction::EST));
        let individual = individual.build().make(w);

        let squad = TestSquad::builder();
        let squad = squad.position(WorldVec2::new(position.x, position.y));
        let squad = squad.members(vec![0.into()]);
        let squad = squad.orders(orders);
        let squad = squad.build().make();

        let world = TestWorld::builder();
        let world = world.individuals(vec![individual]);
        let world = world.squads(vec![squad]);
        let world = world.build().make(&w);

        world
    }
}
