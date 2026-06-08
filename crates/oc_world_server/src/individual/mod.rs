use derive_more::Constructor;
use glam::Vec2;
use oc_individual::{
    Gesture, IndividualIndex, Update,
    behavior::{Behavior, Intent},
    order::Order,
};
use oc_physics::Force;
use oc_root::{Client, physics::MetersSeconds};
use oc_utils::d2::Direction;

use crate::{runner, utils::context::Context};

mod move_;
pub mod physics;
pub mod update;

#[derive(Constructor)]
pub struct Processor<'a, E: Client> {
    ctx: &'a Context<E>,
    i: IndividualIndex,
}

impl<'a, E: Client> Processor<'a, E> {
    pub fn step(self) -> Vec<runner::update::Update> {
        tracing::trace!(name="individual-step", i=?self.i);
        let mut updates = vec![];

        let state = &self.ctx.state;
        let world = state.world();
        let individual = world.individual(self.i);
        if !individual.status.can_step() {
            tracing::trace!(name="individual-step-cant-step", i=?self.i);
            return vec![];
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
            let member = world.individual(member_i);
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

    fn distribute(&self) -> Vec<(IndividualIndex, Vec<Order>)> {
        let state = &self.ctx.state;
        let index = state.indexes();
        let world = state.world();

        let squad_i = index.individual_squad(self.i);
        let squad = world.squad(squad_i);
        // TODO: test if is the squad leader (if its too CPU consuming, manage boolean in individual ?)
        if squad.members.first() != Some(&self.i) {
            tracing::trace!(name="individual-step-distribute-not-leader", i=?self.i);
            return vec![];
        }
        let Some(order) = squad.orders.first() else {
            return vec![];
        };

        let mut distribution = Vec::with_capacity(squad.members.len());
        for member in &squad.members {
            tracing::trace!(name="individual-step-distribute-to", i=?self.i, member=?member);
            distribution.push((*member, vec![order.clone()])) // FIXME BS NOW: real orders (according to squad form, situation, order, etc.)
        }

        distribution
    }

    fn decide(&self) -> Intent {
        let state = &self.ctx.state;
        let world = state.world();
        let individual = world.individual(self.i);
        let order = individual.orders.first();

        match individual.can_follow_order() {
            // TODO: things which can prohibe follow order
            true => match order {
                None => Intent::Idle(Direction::NORTH),
                Some(Order::Idle) => Intent::Idle(Direction::NORTH),
                Some(Order::MoveTo(position)) => Intent::MoveTo(position.clone()),
            },
            false => individual.intent.clone(),
        }
    }

    fn act(&self, intent: &Intent) -> Behavior {
        let state = &self.ctx.state;
        let world = state.world();
        let individual = world.individual(self.i);

        match intent {
            Intent::Idle(direction) => Behavior::Idle(direction.clone()),
            Intent::MoveTo(position) => {
                // TODO: path finding, etc
                let from = Vec2::new(individual.position[0], individual.position[1]);
                let to = Vec2::new(position.x, position.y);
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
                // FIXME BSN NOW: speed (according to behavior, tile)
                vec![Force::Translation(direction.into(), MetersSeconds(1.0))]
            }
        }
    }
}
