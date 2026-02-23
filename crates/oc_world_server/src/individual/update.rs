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
                Listening::Tile(vec![individual.tile])
            }
            Update::UpdateTile(xy) => {
                let before = individual.tile;
                individual.tile = xy;
                indexes.update_individual_xy(i, before, xy);
                Listening::Tile(vec![before, xy])
            }
            Update::UpdateRegion(region) => {
                let before = individual.region;
                individual.region = region;
                // FIXME BS NOW: send entire individual to listener (which are not listening old region)
                todo!()
            }
            Update::UpdateBehavior(behavior) => {
                individual.behavior = behavior.clone();
                Listening::Tile(vec![individual.tile])
            }
            Update::PushForce(force) => {
                individual.forces.push(force);
                Listening::Tile(vec![individual.tile])
            }
            Update::RemoveForce(force) => {
                individual.forces.retain(|f| f != &force);
                Listening::Tile(vec![individual.tile])
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
