use oc_individual::{IndividualIndex, Update, behavior::Intent, order::Order};
use oc_utils::d2::{AlmostEqual, Direction};

use super::{PATH_STEP_TOLERANCE, POSITION_TOLERANCE, Processor, situation::Situation};
use crate::runner;

impl Processor<'_> {
    /// Determine if current order (or order step) is accomplished
    pub fn accomplished(&self, _situation: &Situation) -> Option<Vec<runner::update::Update>> {
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
            Order::MoveTo(position) | Order::MoveFastTo(position) | Order::SneakTo(position) => {
                if position.almost_equal(individual.position, POSITION_TOLERANCE) {
                    tracing::trace!(name = "individual-step-accomplished-move-to-finished", i = ?self.i);
                    updates = Some(accomplished_updates(self.i, direction));
                }
            }
            // Defend and hide never finish
            Order::Defend(_) | Order::Hide(_) => {}
            // FIXME: must finish when ammo low
            Order::Suppress(_) => {}
            Order::Engage(individual) => {
                if !self.world.individual(*individual).status.is_target() {
                    tracing::trace!(name = "individual-step-accomplished-engage-finished", i=?self.i, i2=?individual);
                    updates = Some(accomplished_updates(self.i, direction));
                }
            }
        };

        if updates.is_some() {
            return updates;
        }

        // Test move intents to trigger "next step" when step reached
        match &individual.intent {
            // Idle/Defend/Hide/Engage/Suppress never finish
            Intent::Idle(_)
            | Intent::Defend(_)
            | Intent::Hide(_)
            | Intent::Engage(_)
            | Intent::Suppress(_) => {}
            Intent::MoveTo(_, move_path)
            | Intent::MoveFastTo(_, move_path)
            | Intent::SneakTo(_, move_path) => {
                let Some(next) = move_path.iter().next() else {
                    tracing::trace!(name = "individual-step-accomplished-intent-move-to-no-next", i=?self.i);
                    return updates;
                };

                if next.almost_equal(individual.position, PATH_STEP_TOLERANCE) {
                    let update = Update::MoveStepAccomplished;
                    let update = runner::update::Update::UpdateIndividual(self.i, update);
                    updates.get_or_insert(vec![]).push(update);
                }
            }
        }

        updates
    }
}
