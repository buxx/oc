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
    let (region_before, region_after) = {
        let mut world = state.world_mut();
        let mut indexes = state.indexes_mut();
        let individual = world.individual_mut(i);
        let region_before = individual.region;

        match update.clone() {
            Update::UpdatePosition(position) => {
                individual.position = position;
            }
            Update::UpdateTile(tile) => {
                let before = individual.tile;
                individual.tile = tile;
                indexes.update_individual_tile(i, before, tile);
            }
            Update::UpdateRegion(region) => {
                let before = individual.region;
                individual.region = region;
                indexes.update_individual_region(i, before, region);
            }
            Update::UpdateBehavior(behavior) => {
                individual.behavior = behavior.clone();
            }
            Update::PushForce(force) => {
                individual.forces.push(force);
            }
            Update::RemoveForce(force) => {
                individual.forces.retain(|f| f != &force);
            }
        }

        (region_before, individual.region)
    };

    broadcast(
        state,
        Listening::Regions(vec![region_before.into()]),
        vec![network::Individual::Update(i, update)],
        output,
    );

    if region_before != region_after {
        let world = state.world();
        let individual = world.individual(i).clone();

        broadcast(
            state,
            Listening::Regions(vec![]),
            vec![network::Individual::Insert(i, individual)],
            output,
        );
    }
}

fn broadcast(
    state: &Arc<State>,
    filter: Listening,
    messages: Vec<network::Individual>,
    output: &Sender<(Endpoint, ToClient)>,
) {
    state
        .listeners()
        .find(filter)
        .into_iter()
        .for_each(|endpoint| {
            messages
                .iter()
                .for_each(|message| output.send((endpoint, message.clone().into())).unwrap()) // TODO
        });
}
