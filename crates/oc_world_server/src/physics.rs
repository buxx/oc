use derive_more::Constructor;
use oc_geo::{
    Geo, UpdateGeo,
    region::{Region, RegionXy},
    tile::TileXy,
};
use oc_individual::IndividualIndex;
use oc_physics::{Force, Laws, Physic, UpdatePhysic, update::Update};
use oc_utils::collections::WithIds;
use oc_world::World;

use crate::{
    index::{self, IntoIndexEffect},
    network::{IntoNetworkInsert, IntoNetworkUpdate},
    routing::Listening,
    utils::{context::Context, subject::IntoSubject},
};

#[derive(Constructor)]
pub struct Processor<'x> {
    ctx: &'x Context,
}

impl<'x> Processor<'x> {
    pub fn step(&self, i: usize) {
        let subjects = self.step_for(i, |w| w.individuals().with_ids());
        let effects = self.write(subjects, |w, i| w.individual_mut(i));

        for (i, (region, effect), update) in effects {
            // Broadcast the update
            let filter = Listening::Regions(vec![region.into()]);
            let messages = vec![update.into_network_update(i)];
            self.ctx.broadcast(filter, messages);

            if let Some(effect) = effect {
                // Update indexes
                {
                    let mut indexes = self.ctx.state.indexes_mut();
                    indexes.react(effect.into_index_effect(i));
                }

                // Broadcast to new listener if required
                if let Effect::Region { before: _, after } = effect {
                    let world = self.ctx.state.world();
                    let subject = i.into_subject(&world);

                    tracing::trace!(name="subject-update-write-broadast-insert", i=?i);
                    let filter = Listening::Regions(vec![after.into()]);
                    let messages = vec![subject.into_network_insert(i)];
                    self.ctx.broadcast(filter, messages);
                }
            }
        }
    }

    fn step_for<F, I, T>(&self, i: usize, all: F) -> Vec<(I, Vec<Update>)>
    where
        for<'a> F: Fn(&'a World) -> Vec<(I, &'a T)>,
        I: std::fmt::Debug + Copy,
        T: Physic + Geo + Region,
    {
        let world = self.ctx.state.world();
        let all = all(&world);
        let size = (all.len() as f32 / self.ctx.cpus as f32).ceil() as usize;
        if size == 0 {
            return vec![];
        };
        let chunks = all.chunks(size).collect::<Vec<_>>();
        let chunk = chunks.get(i);
        let Some(chunk) = chunk else { return vec![] };
        let tiles = |xy| world.tile(TileXy(xy));

        chunk
            .into_iter()
            .map(|(i, subject)| {
                let laws = Laws::default();
                let (new_position, forces) = oc_physics::step(&laws, *subject, tiles);
                tracing::trace!(name="physics-subject", i=?i, new_position=?new_position, forces=?forces);
                let updates = changes(i, *subject, &new_position, &forces);
                (*i, updates)
            })
            .collect::<Vec<_>>()
    }

    fn write<F, I, T>(
        &self,
        subjects: Vec<(I, Vec<Update>)>,
        get: F,
    ) -> Vec<(I, (RegionXy, Option<Effect>), Update)>
    where
        F: for<'a> Fn(&'a mut World, I) -> &'a mut T,
        I: std::fmt::Debug + Copy,
        T: Clone + Physic + UpdatePhysic + Geo + UpdateGeo + Region,
    {
        subjects
            .into_iter()
            .flat_map(|(i, updates)| updates.into_iter().map(move |update| (i, update)))
            .map(|(i, update)| {
                let mut world = self.ctx.state.world_mut();
                let subject = get(&mut world, i);
                let effect = write(&update, subject);

                (i, effect, update)
            })
            .collect()
    }
}

pub fn changes<I, T>(i: I, individual: &T, position: &[f32; 2], forces: &Vec<Force>) -> Vec<Update>
where
    I: std::fmt::Debug,
    T: Physic + Geo + Region,
{
    let mut updates = vec![];

    if individual.position() != position {
        updates.push(Update::UpdatePosition(*position));

        let tile: TileXy = position.clone().into();
        let region: RegionXy = tile.into();

        if individual.tile() != &tile {
            updates.push(Update::UpdateTile(tile));
        }

        if individual.region() != &region {
            updates.push(Update::UpdateRegion(region));
        }
    }

    for force in individual.forces() {
        if !forces.contains(force) {
            updates.push(Update::RemoveForce(force.clone()));
        }
    }

    for force in forces {
        if !individual.forces().contains(force) {
            updates.push(Update::PushForce(force.clone()));
        }
    }

    tracing::trace!(name="physics-individual-updates", i=?i, updates=?updates);

    updates
}

pub fn write<T>(update: &Update, subject: &mut T) -> (RegionXy, Option<Effect>)
where
    T: Clone + Physic + UpdatePhysic + Geo + UpdateGeo + Region,
{
    let tile = subject.tile().clone();
    let region = subject.region().clone();

    let effect = match update {
        Update::UpdatePosition(position) => {
            subject.set_position(*position);
            None
        }
        Update::UpdateTile(tile_) => {
            subject.set_tile(*tile_);
            Some(Effect::Tile {
                before: tile,
                after: *tile_,
            })
        }
        Update::UpdateRegion(region_) => {
            subject.set_region(*region_);
            Some(Effect::Region {
                before: region,
                after: *region_,
            })
        }
        Update::PushForce(force) => {
            subject.push_force(force.clone());
            None
        }
        Update::RemoveForce(force) => {
            subject.remove_force(&force);
            None
        }
    };

    (region, effect)
}

#[derive(Debug, Clone)]
pub enum Effect {
    Tile { before: TileXy, after: TileXy },
    Region { before: RegionXy, after: RegionXy },
}

impl IntoIndexEffect<IndividualIndex> for Effect {
    fn into_index_effect(&self, i: IndividualIndex) -> index::Effect {
        let effect = index::IndividualEffect::Physic(self.clone());
        index::Effect::Individual(i, effect)
    }
}
