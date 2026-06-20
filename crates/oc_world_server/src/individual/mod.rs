use derive_more::Constructor;
use glam::Vec2;
use oc_individual::{
    Gesture, IndividualIndex, Update,
    behavior::{Behavior, Intent, MovePath},
    order::Order,
};
use oc_physics::Force;
use oc_root::physics::MetersSeconds;
use oc_utils::{d2::Direction, number::almost_equal};
use oc_world::World;

use crate::{index::Indexes, runner};

mod move_;
pub mod physics;
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
            tracing::trace!(name = "individual-step-accomplished-updates", updates=?updates);
            return updates;
        }

        let distribute = self.distribute();
        let intent = self.decide();
        let behavior = self.act(&intent);
        let gesture = self.gesture(&behavior);
        let forces = self.forces(&behavior);

        tracing::trace!(
            name = "individual-step-with",
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
                let update = runner::update::Update::UpdateIndividual(self.i, update);
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

        tracing::trace!(name = "individual-step-updates", updates=?updates);
        updates
    }

    fn is_squad_leader(&self) -> bool {
        let squad_i = self.index.individual_squad(self.i);
        let squad = self.world.squad(squad_i);
        // TODO: test if is the squad leader (if its too CPU consuming, manage boolean in individual ?)
        squad.members.first() == Some(&self.i)
    }

    fn accomplished(&self) -> Option<Vec<runner::update::Update>> {
        let individual = self.world.individual(self.i);
        let Some(order) = individual.orders.first() else {
            tracing::trace!(name = "individual-step-accomplished-no-order", i=?self.i);
            return None;
        };
        let squad_i = self.index.individual_squad(self.i);
        let squad = self.world.squad(squad_i);
        let is_squad_leader = self.is_squad_leader();
        let mut updates = None;

        match order {
            Order::Idle => {
                // TODO: strange behavior than Idle disapear instantly ?
                tracing::trace!(name = "individual-step-accomplished-squad-leader-idle-finished", i=?self.i);

                let update_i1 = Update::Accomplished;
                let update_i1 = runner::update::Update::UpdateIndividual(self.i, update_i1);
                let update_i2 = Update::SetIntent(Intent::Idle(Direction::NORTH)); // TODO direction
                let update_i2 = runner::update::Update::UpdateIndividual(self.i, update_i2);

                match is_squad_leader {
                    true => {
                        // FIXME BS NOW: Argh refacto
                        // FIXME: not sure than squal leader idle finished indicate all squad accomplished ...
                        let update_s1 = oc_individual::squad::Update::Accomplished;
                        let update_s1 = runner::update::Update::UpdateSquad(squad_i, update_s1);
                        let orders = squad.orders.clone().into_iter().skip(1).collect();
                        let update_s2 = oc_individual::squad::Update::SetOrders(orders);
                        let update_s2 = runner::update::Update::UpdateSquad(squad_i, update_s2);
                        updates = Some(vec![update_i1, update_i2, update_s1, update_s2]);
                    }
                    false => {
                        updates = Some(vec![update_i1, update_i2]);
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

                    let update_i1 = Update::Accomplished;
                    let update_i1 = runner::update::Update::UpdateIndividual(self.i, update_i1);
                    let update_i2 = Update::SetIntent(Intent::Idle(Direction::NORTH)); // TODO direction
                    let update_i2 = runner::update::Update::UpdateIndividual(self.i, update_i2);

                    match is_squad_leader {
                        true => {
                            // FIXME BS NOW: Argh refacto
                            // FIXME: not sure than squal leader idle finished indicate all squad accomplished ...
                            // Must wait all memeber finished associated order.
                            let update_s1 = oc_individual::squad::Update::Accomplished;
                            let update_s1 = runner::update::Update::UpdateSquad(squad_i, update_s1);
                            let orders = squad.orders.clone().into_iter().skip(1).collect();
                            let update_s2 = oc_individual::squad::Update::SetOrders(orders);
                            let update_s2 = runner::update::Update::UpdateSquad(squad_i, update_s2);
                            updates = Some(vec![update_i1, update_i2, update_s1, update_s2]);
                        }
                        false => {
                            updates = Some(vec![update_i1, update_i2]);
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
                    tracing::trace!(name = "individual-step-accomplished-intent-move-to-no-next");
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

    fn distribute(&self) -> Vec<(IndividualIndex, Vec<Order>)> {
        let squad_i = self.index.individual_squad(self.i);
        let squad = self.world.squad(squad_i);
        // TODO: test if is the squad leader (if its too CPU consuming, manage boolean in individual ?)
        if !self.is_squad_leader() {
            tracing::trace!(name="individual-step-distribute-not-leader", i=?self.i);
            return vec![];
        }
        let Some(order) = squad.orders.first() else {
            return vec![];
        };

        let mut distribution = Vec::with_capacity(squad.members.len());
        for member in &squad.members {
            tracing::trace!(name="individual-step-distribute-to", i=?self.i, member=?member);
            distribution.push((*member, vec![order.clone()])) // FIXME: real orders (according to squad form, situation, order, etc.)
        }

        distribution
    }

    fn decide(&self) -> Intent {
        let individual = self.world.individual(self.i);
        let order = individual.orders.first();

        match individual.can_follow_order() {
            // TODO: things which can prohibe follow order
            true => match order {
                None | Some(Order::Idle) => Intent::Idle(Direction::NORTH),
                Some(Order::MoveTo(position)) => {
                    // FIXME BS NOW: do not recompute each time the path, use cached one and, regurlarly compute new one
                    // perdiodic or when collision ?
                    let from = Vec2::new(individual.position[0], individual.position[1]);
                    let to = Vec2::new(position.x, position.y);
                    let path = self.world.navmesh.path(from, to);
                    match path {
                        Some(path) => Intent::MoveTo(position.clone(), MovePath::from(path)),
                        None => Intent::Idle(Direction::NORTH),
                    }
                }
            },
            false => individual.intent.clone(),
        }
    }

    fn act(&self, intent: &Intent) -> Behavior {
        let individual = self.world.individual(self.i);

        match intent {
            Intent::Idle(direction) => Behavior::Idle(direction.clone()),
            Intent::MoveTo(_, path) => {
                let Some(next) = path.iter().next() else {
                    tracing::trace!(name = "individual-step-act-move-no-next");
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
