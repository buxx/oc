use oc_individual::{IndividualIndex, Update, network};
use oc_root::Client;

use crate::{routing::Listening, utils::context::Context};

pub fn write<E: Client>(ctx: &Context<E>, update: Update, i: IndividualIndex) {
    let mut world = ctx.state.world_mut();
    let individual = world.individual_mut(i);

    match &update {
        Update::SetBehavior(behavior) => {
            individual.behavior = *behavior;
        }
        Update::SetForces(forces) => {
            individual.forces = forces.clone();
        }
    }

    let region = individual.region;
    let filter = Listening::Regions(vec![region]);
    let messages = vec![network::Individual::Update(i, update)];
    ctx.broadcast(filter, messages);
}
