use std::sync::{Arc, mpsc::Sender};

use message_io::network::Endpoint;
use oc_individual::{IndividualIndex, Update, network};
use oc_network::ToClient;

use crate::{routing::Listening, state::State};

pub fn write(
    update: Update,
    i: IndividualIndex,
    state: &Arc<State>,
    output: &Sender<(Endpoint, ToClient)>,
) {
    let filter = {
        let mut world = state.world_mut();
        let mut indexes = state.indexes_mut();
        let individual = world.individual_mut(i);

        match update.clone() {
            Update::UpdatePosition(position) => {
                individual.position = position;
                Listening::TileXy(vec![individual.tile])
            }
            Update::UpdateXy(xy) => {
                let before = individual.tile;
                individual.tile = xy;
                indexes.update_individual_xy(i, before, xy);
                Listening::TileXy(vec![before, xy])
            }
            Update::UpdateBehavior(behavior) => {
                individual.behavior = behavior.clone();
                Listening::TileXy(vec![individual.tile])
            }
            Update::PushForce(force) => {
                individual.forces.push(force);
                Listening::TileXy(vec![individual.tile])
            }
            Update::RemoveForce(force) => {
                individual.forces.retain(|f| f != &force);
                Listening::TileXy(vec![individual.tile])
            }
        }
    };

    let message = network::Individual::Update(i, update);
    let msg: ToClient = message.into();

    state
        .listeners()
        .find(filter)
        .into_iter()
        .for_each(|endpoint| output.send((endpoint, msg.clone())).unwrap()); // TODO
}
