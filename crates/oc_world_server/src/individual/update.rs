use std::sync::{Arc, mpsc::Sender};

use message_io::network::Endpoint;
use oc_individual::{IndividualIndex, Update, network};
use oc_network::ToClient;

use crate::{index, physics, routing::Listening, state::State};

pub fn write(
    update: Update,
    i: IndividualIndex,
    state: &Arc<State>,
    output: &Sender<(Endpoint, ToClient)>,
) {
    let mut world = state.world_mut();
    let individual = world.individual_mut(i);

    match update {
        Update::Physics(update) => {
            // FIXME: its a good idea to permit push pysics from individual updates ? This is a BIG call below ...
            // Think about another way.
            let (region, effect) = physics::write(&update, individual);

            if let Some(effect) = effect {
                let effect = index::IndividualEffect::Physic(effect);
                let mut indexes = state.indexes_mut();
                indexes.react(index::Effect::Individual(i, effect));
            }

            physics::apply(
                state,
                output,
                i,
                region,
                effect,
                update,
                |w, i| w.individual(i),
                |i, u| {
                    oc_individual::network::Individual::Update(i, oc_individual::Update::Physics(u))
                },
                |i, ind| oc_individual::network::Individual::Insert(i, ind.clone()),
            );
        }
        Update::UpdateBehavior(behavior) => {
            // FIXME: refactor broadcast effects method (need to broadcast it too ! not only physics...)
            individual.behavior = behavior;

            crate::network::broadcast(
                state,
                Listening::Regions(vec![individual.region().clone().into()]),
                vec![network::Individual::Update(i, update)],
                output,
            );
        }
    }

    // (region_before, individual.region)
    // };

    // crate::network::broadcast(
    //     state,
    //     Listening::Regions(vec![region_before.into()]),
    //     vec![network::Individual::Update(i, update)],
    //     output,
    // );

    // if region_before != region_after {
    //     let world = state.world();
    //     let individual = world.individual(i).clone();

    //     tracing::trace!(name="individual-update-write-broadast-insert", i=?i);
    //     crate::network::broadcast(
    //         state,
    //         Listening::Regions(vec![]),
    //         vec![network::Individual::Insert(i, individual)],
    //         output,
    //     );
    // }
}
