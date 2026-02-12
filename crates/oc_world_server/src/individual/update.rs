use std::sync::Arc;

use oc_individual::{IndividualIndex, Update};

use crate::{network::Network, state::State};

pub fn write(update: &Update, i: IndividualIndex, state: &Arc<State>, network: &Arc<Network>) {
    {
        let mut world = state.world_mut();
        let mut indexes = state.indexes_mut();
        let individual = world.individual_mut(i);

        match update {
            Update::UpdateXy(xy) => {
                let before = individual.xy;
                individual.xy = *xy;
                indexes.update_individual_xy(i, before, *xy);
            }
            Update::UpdateBehavior(behavior) => {
                individual.behavior = behavior.clone();
            }
        }
    }

    let msg = oc_individual::network::IndividualMessage::Effect(i, update.clone());
    network.broadcast(msg.into());
}
