use std::sync::{Arc, mpsc::Sender};

use derive_more::Constructor;
use message_io::network::Endpoint;
use oc_geo::tile::TileXy;
use oc_individual::{Individual, IndividualIndex};
use oc_network::ToClient;
use oc_physics::Laws;

use crate::{individual, state::State};

#[derive(Constructor)]
pub struct Processor {
    cpus: usize,
    state: Arc<State>,
    output: Sender<(Endpoint, ToClient)>,
}

impl Processor {
    pub fn step(&self, i: usize) {
        self.individuals(i);
    }

    fn individuals(&self, i: usize) {
        let updates: Vec<(IndividualIndex, Vec<oc_individual::Update>)> = {
            let world = self.state.world();
            let all: Vec<(usize, &Individual)> =
                world.individuals().into_iter().enumerate().collect();
            let size = (all.len() as f32 / self.cpus as f32).ceil() as usize;
            let chunks = all.chunks(size).collect::<Vec<_>>();
            let chunk = chunks.get(i);
            let Some(chunk) = chunk else { return };
            let tiles = |xy| world.tile(TileXy(xy));

            chunk
                .into_iter()
                .map(|(i, individual)| {
                    let i = IndividualIndex(*i as u64);
                    let laws = Laws::default();
                    let (new_position, forces) = oc_physics::step(&laws, individual, tiles);
                    tracing::trace!(name="physics-individual", i=?i, new_position=?new_position, forces=?forces);
                    let updates =
                        individual::physics::changes(i, individual, &new_position, &forces);
                    (i, updates)
                })
                .collect::<Vec<_>>()
        };

        updates.into_iter().for_each(|(i, updates)| {
            updates.into_iter().for_each(|update| {
                individual::update::write(update, i, &self.state, &self.output);
            });
        });
    }
}
