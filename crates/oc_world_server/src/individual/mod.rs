use derive_more::Constructor;
use glam::Vec2;
use oc_individual::{
    Gesture, IndividualIndex, Update,
    behavior::{Behavior, Intent, MovePath},
    order::Order,
};
use oc_physics::Force;
use oc_root::{physics::MetersSeconds, y::V};
use oc_utils::{d2::Direction, number::almost_equal};
use oc_world::World;

use crate::{index::Indexes, runner};

pub mod update;

const POSITION_TOLERANCE: f32 = 3.0;

#[derive(Constructor)]
pub struct Processor<'a> {
    world: &'a World,
    index: &'a Indexes,
    i: IndividualIndex,
}

// TODO: a lot of repetition (and locking) due to function split. Find another solution (perf problem ?)
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

        let distribute = self.distribute();
        let intent = self.decide();
        let behavior = self.act(&intent);
        let gesture = self.gesture(&behavior);
        let forces = self.forces(&behavior);

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

        // TODO: macro for repetitives if bellow ?
        if intent != individual.intent {
            let update = Update::SetIntent(intent);
            let update = runner::update::Update::UpdateIndividual(self.i, update);
            updates.push(update);
        }

        if behavior != individual.behavior {
            let update = Update::SetBehavior(behavior);
            let update = runner::update::Update::UpdateIndividual(self.i, update);
            updates.push(update);
        }

        if gesture != individual.gesture {
            let update = Update::SetGesture(gesture);
            let update = runner::update::Update::UpdateIndividual(self.i, update);
            updates.push(update);
        }

        if forces != individual.forces {
            let update = Update::SetForces(forces);
            let update = runner::update::Update::UpdateIndividual(self.i, update);
            updates.push(update);
        }

        tracing::trace!(name = "individual-step-updates", i=?self.i, updates=?updates);
        updates
    }

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

        match order {
            Order::Idle => {
                // TODO: strange behavior than Idle disapear instantly ?
                tracing::trace!(name = "individual-step-accomplished-squad-leader-idle-finished", i=?self.i);

                let i_accomplish = Update::Accomplished;
                let i_accomplish = runner::update::Update::UpdateIndividual(self.i, i_accomplish);
                let i_idle = Update::SetIntent(Intent::Idle(direction));
                let i_idle = runner::update::Update::UpdateIndividual(self.i, i_idle);

                match is_squad_leader {
                    true => {
                        // FIXME BS NOW: Argh refacto
                        let accomplish = oc_individual::squad::Update::Accomplished;
                        let accomplish = runner::update::Update::UpdateSquad(squad_i, accomplish);
                        let orders = squad.orders.clone().into_iter().skip(1).collect();
                        let orders = oc_individual::squad::Update::SetOrders(orders);
                        let orders = runner::update::Update::UpdateSquad(squad_i, orders);
                        updates = Some(vec![i_accomplish, i_idle, accomplish, orders]);
                    }
                    false => {
                        updates = Some(vec![i_accomplish, i_idle]);
                    }
                };
            }
            Order::MoveTo(position) => {
                if almost_equal(position.x, individual.position[0], POSITION_TOLERANCE)
                    && almost_equal(position.y, individual.position[1], POSITION_TOLERANCE)
                {
                    tracing::trace!(
                        name = "individual-step-accomplished-squad-leader-move-to-finished", i=?self.i
                    );

                    let i_accomplish = Update::Accomplished;
                    let i_accomplish =
                        runner::update::Update::UpdateIndividual(self.i, i_accomplish);
                    let idle = Update::SetIntent(Intent::Idle(direction));
                    let idle = runner::update::Update::UpdateIndividual(self.i, idle);

                    match is_squad_leader {
                        true => {
                            // FIXME BS NOW: Argh refacto
                            // FIXME: Must wait all memeber finished associated order.
                            let accomplish = oc_individual::squad::Update::Accomplished;
                            let accomplish =
                                runner::update::Update::UpdateSquad(squad_i, accomplish);
                            let orders = squad.orders.clone().into_iter().skip(1).collect();
                            let orders = oc_individual::squad::Update::SetOrders(orders);
                            let orders = runner::update::Update::UpdateSquad(squad_i, orders);
                            updates = Some(vec![i_accomplish, idle, accomplish, orders]);
                        }
                        false => {
                            updates = Some(vec![i_accomplish, idle]);
                        }
                    }
                }
            }
        };

        if updates.is_some() {
            return updates;
        }

        match &individual.intent {
            Intent::Idle(_) => {} // Never end
            Intent::MoveTo(_, move_path) => {
                let Some(next) = move_path.iter().next() else {
                    tracing::trace!(name = "individual-step-accomplished-intent-move-to-no-next", i=?self.i);
                    return updates;
                };

                if almost_equal(next[0], individual.position[0], POSITION_TOLERANCE)
                    && almost_equal(next[1], individual.position[1], POSITION_TOLERANCE)
                {
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

        // TODO: test if is the squad leader (if its too CPU consuming, manage boolean in individual ?)
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
        let reference = Vec2::new(leader.position[0], leader.position[1]);
        let count = squad.actives as usize;
        let direction = gesture.direction();
        let angle = direction.angle();

        let positions =
            squad
                .formation
                .positions(&self.world.w, V::Server, reference, angle, count);
        tracing::trace!(name="individual-step-distribute-formation", i=?self.i, squad_i=?squad_i, reference=?reference, direction=?direction, angle=?angle, positions=?positions);

        for (member, position) in squad
            .members
            .iter()
            .zip(positions)
            // Skip leader as already disributed before
            .skip(1)
        {
            // According to order, choose appropriate order to distribute (move if move, move fast if move fast, etc.)
            let orders = match order {
                Some(order) => match order {
                    Order::Idle => vec![Order::MoveTo(position.into())],
                    Order::MoveTo(_) => vec![Order::MoveTo(position.into())],
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

    fn decide(&self) -> Intent {
        let individual = self.world.individual(self.i);
        let order = individual.orders.first();
        let direction = individual.gesture.direction();

        let intent = match individual.can_follow_order() {
            // TODO: things which can prohibe follow order
            true => {
                match order {
                    None | Some(Order::Idle) => Intent::Idle(direction),
                    Some(Order::MoveTo(position)) => {
                        // TODO: think about a way to cache that ? Or not if don't take too much CPU
                        let from = Vec2::new(individual.position[0], individual.position[1]);
                        let to = Vec2::new(position.x, position.y);
                        let path = self.world.navmesh.path(from, to);
                        match path {
                            Some(path) => Intent::MoveTo(position.clone(), MovePath::from(path)),
                            None => Intent::Idle(direction),
                        }
                    }
                }
            }
            false => individual.intent.clone(),
        };

        tracing::trace!(name="individual-step-decide", i=?self.i, order=?order, intent=?intent);
        intent
    }

    fn act(&self, intent: &Intent) -> Behavior {
        let individual = self.world.individual(self.i);

        match intent {
            Intent::Idle(direction) => Behavior::Idle(direction.clone()),
            Intent::MoveTo(_, path) => {
                let Some(next) = path.iter().next() else {
                    tracing::trace!(name = "individual-step-act-move-no-next", i=?self.i);
                    return Behavior::Idle(Direction::NORTH); // should not happen
                };

                let from = Vec2::new(individual.position[0], individual.position[1]);
                let to = Vec2::new(next[0], next[1]);
                let direction = (to - from).normalize_or_zero();
                Behavior::Walk(Direction::from(direction))
            }
        }
    }

    fn gesture(&self, behavior: &Behavior) -> Gesture {
        match behavior {
            Behavior::Idle(direction) => Gesture::Idle(direction.clone()),
            Behavior::Walk(direction) => Gesture::Walking(direction.clone()),
        }
    }

    fn forces(&self, behavior: &Behavior) -> Vec<Force> {
        match behavior {
            Behavior::Idle(_) => vec![],
            Behavior::Walk(direction) => {
                // FIXME BSN NOW: z (tile z)
                let direction = Vec2::from(direction.clone()).extend(0.);
                // dbg!(&direction);
                // FIXME BSN NOW: speed (according to behavior, tile)
                vec![Force::Translation(direction.into(), MetersSeconds(1.0))]
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use glam::{Vec2, Vec3};
    use oc_individual::{
        Gesture, IndividualIndex,
        behavior::{Behavior, Intent},
        order::Order,
        squad::SquadIndex,
    };
    use oc_root::{WorldConfig, physics::Meters};
    use oc_utils::d2::{Direction, Position};
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
        let individual_1_position = Vec3::new(100., 100., 0.);
        let individual_2_position = Vec3::new(90., 110., 0.);
        let squad_position = Vec2::new(individual_1_position.x, individual_1_position.y);
        let move_to_position = Position::new(150., 100.);
        let move_to_order = Order::MoveTo(move_to_position);
        // expected
        let expected_individual_1_move_to_position = Position::new(150., 100.);
        let expected_individual_2_move_to_position = Position::new(100., 110.);

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
        let individual_1_position = Vec3::new(100., 100., 0.);
        let individual_2_position = Vec3::new(90., 110., 0.);
        let squad_position = Vec2::new(individual_1_position.x, individual_1_position.y);
        // expected
        let expected_individual_2_move_to_position = Position::new(100., 110.);

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
        let position = Vec3::new(100., 100., 0.);

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
        let position = Vec3::new(100., 100., 0.);

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
        individual_1_position: Vec3,
        individual_2_position: Vec3,
        squad_position: Vec2,
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
    fn one_individual_world(w: &WorldConfig, position: Vec3, orders: Vec<Order>) -> World {
        let individual = TestIndividual::builder();
        let individual = individual.position(position);
        let individual = individual
            .gesture(Gesture::Idle(Direction::EST)) // Gesture & Behavior & Intent are important
            .behavior(Behavior::Idle(Direction::EST)) // to conditionate the .step() response
            .intent(Intent::Idle(Direction::EST));
        let individual = individual.build().make(w);

        let squad = TestSquad::builder();
        let squad = squad.position(Vec2::new(position.x, position.y));
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
