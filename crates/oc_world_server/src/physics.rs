use std::sync::{Arc, mpsc::Sender};

use derive_more::Constructor;
use message_io::network::Endpoint;
use oc_geo::tile::TileXy;
use oc_individual::{Individual, IndividualIndex};
use oc_network::ToClient;
use oc_physics::{Force, Laws, Physic};
use oc_projectile::{Projectile, ProjectileId};

use crate::{individual, projectile, state::State};

#[derive(Constructor)]
pub struct Processor {
    cpus: usize,
    state: Arc<State>,
    output: Sender<(Endpoint, ToClient)>,
}

impl Processor {
    // FIXME: refactor (trait, macro, etc)
    pub fn step(&self, i: usize) {
        let individuals = self.individuals(i);
        let projectiles = self.projectiles(i);

        individuals.into_iter().for_each(|(i, updates)| {
            updates.into_iter().for_each(|update| {
                individual::update::write(update, i, &self.state, &self.output);
            });
        });

        projectiles.into_iter().for_each(|(i, updates)| {
            updates.into_iter().for_each(|update| {
                projectile::update::write(update, i, &self.state, &self.output);
            });
        });
    }

    fn individuals(&self, i: usize) -> Vec<(IndividualIndex, Vec<oc_individual::Update>)> {
        let world = self.state.world();
        let all: Vec<(usize, &Individual)> = world.individuals().into_iter().enumerate().collect();
        let size = (all.len() as f32 / self.cpus as f32).ceil() as usize;
        if size == 0 {
            return vec![];
        };
        let chunks = all.chunks(size).collect::<Vec<_>>();
        let chunk = chunks.get(i);
        let Some(chunk) = chunk else { return vec![] };
        let tiles = |xy| world.tile(TileXy(xy));

        chunk
            .into_iter()
            .map(|(i, individual)| {
                let i = IndividualIndex(*i as u64);
                let laws = Laws::default();
                let (new_position, forces) = oc_physics::step(&laws, *individual, tiles);
                tracing::trace!(name="physics-individual", i=?i, new_position=?new_position, forces=?forces);
                let updates =
                    individual::physics::changes(i, individual, &new_position, &forces);
                (i, updates)
            })
            .collect::<Vec<_>>()
    }

    fn projectiles(&self, i: usize) -> Vec<(ProjectileId, Vec<oc_projectile::Update>)> {
        let world = self.state.world();
        let all: Vec<(usize, (&ProjectileId, &Projectile))> =
            world.projectiles().into_iter().enumerate().collect();
        let size = (all.len() as f32 / self.cpus as f32).ceil() as usize;
        if size == 0 {
            return vec![];
        };
        let chunks = all.chunks(size).collect::<Vec<_>>();
        let chunk = chunks.get(i);
        let Some(chunk) = chunk else { return vec![] };
        let tiles = |xy| world.tile(TileXy(xy));

        chunk
            .into_iter()
            .map(|(i, (id, projectile))| {
                let laws = Laws::default();
                let (new_position, forces) = oc_physics::step(&laws, *projectile, tiles);
                tracing::trace!(name="physics-projectile", i=?i, new_position=?new_position, forces=?forces);
                let updates =
                    projectile::physics::changes(id, projectile, &new_position, &forces);
                (**id, updates)
            })
            .collect::<Vec<_>>()
    }
}
