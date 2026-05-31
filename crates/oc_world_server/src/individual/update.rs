use oc_individual::{IndividualIndex, Update, network};
use oc_network::ToClient;
use oc_root::Client;

use crate::{routing::Listening, utils::context::Context};

pub fn write<E: Client>(
    ctx: &Context<E>,
    update: Update,
    i: IndividualIndex,
) -> Vec<(Listening, Vec<ToClient>)> {
    let mut world = ctx.state.world_mut();
    let individual = world.individual_mut(i);

    match &update {
        Update::SetBehavior(behavior) => {
            individual.behavior = *behavior;
        }
        Update::SetForces(forces) => {
            individual.forces = forces.clone();
        }
        Update::SetStatus(status) => {
            individual.status = *status;
        }
    }

    let region = individual.region;
    let update = network::Individual::Update(i, update);
    let update = ToClient::Individual(update);
    vec![(Listening::Regions(vec![region]), vec![update])]
}
