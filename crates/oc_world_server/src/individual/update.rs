use oc_individual::{IndividualIndex, Update, network};

use crate::{routing::Listening, utils::context::Context};

pub fn write(ctx: &Context, update: Update, i: IndividualIndex) {
    let mut world = ctx.state.world_mut();
    let individual = world.individual_mut(i);

    match &update {
        Update::SetBehavior(behavior) => {
            individual.behavior = behavior.clone();
        }
        Update::SetForces(forces) => {
            individual.forces = forces.clone();
        }
    }

    let region = individual.region.clone();
    let filter = Listening::Regions(vec![region.clone().into()]);
    let messages = vec![network::Individual::Update(i, update)];
    ctx.broadcast(filter, messages);
}
