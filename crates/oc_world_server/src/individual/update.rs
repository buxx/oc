use std::sync::{Arc, mpsc::Sender};

use message_io::network::Endpoint;
use oc_individual::{IndividualIndex, Update, network};
use oc_network::ToClient;

use crate::{routing::Listening, state::State};

pub fn write(
    update: &Update,
    i: IndividualIndex,
    state: &Arc<State>,
    output: &Sender<(Endpoint, ToClient)>,
) {
    let filter = {
        let mut world = state.world_mut();
        let mut indexes = state.indexes_mut();
        let individual = world.individual_mut(i);

        match update {
            Update::UpdatePosition(xy) => {
                let before = individual.xy;
                individual.xy = *xy;
                indexes.update_individual_xy(i, before, *xy);
                Listening::TileXy(vec![before, *xy])
            }
            Update::UpdateBehavior(behavior) => {
                individual.behavior = behavior.clone();
                Listening::TileXy(vec![individual.xy])
            }
        }
    };

    let message = network::Individual::Effect(i, update.clone());
    let msg: ToClient = message.into();

    state
        .listeners()
        .find(filter)
        .into_iter()
        .for_each(|endpoint| output.send((endpoint, msg.clone())).unwrap()); // TODO
}
