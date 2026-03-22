use derive_more::Constructor;
use oc_geo::{
    Geo, UpdateGeo,
    region::{Region, RegionXy},
    tile::{TileXy, WorldTileIndex},
};
use oc_individual::IndividualIndex;
use oc_physics::{Event, Force, Laws, Physic, UpdatePhysic, update::Update};
use oc_projectile::ProjectileId;
use oc_utils::collections::WithIds;
use oc_world::World;

use crate::{
    index::{self, IntoIndexEffect},
    network::{IntoNetworkForgot, IntoNetworkInsert, IntoNetworkUpdate},
    routing::Listening,
    state::ObjectId,
    utils::{
        context::Context,
        subject::{IntoSubject, IntoSubjectMut},
    },
};

#[derive(Constructor)]
pub struct Processor<'x> {
    ctx: &'x Context,
}

impl<'x> Processor<'x> {
    pub fn step(&self, i: usize) {
        tracing::trace!(name = "physics-step", i = i);

        let (subject_upds, subject_events) = self.step_for(i, |w| w.individuals().with_ids());
        let subject_effects = self.write(subject_upds);

        let (projectile_upds, projectile_events) = self.step_for(i, |w| w.projectiles().with_ids());
        let projectile_effects = self.write(projectile_upds);

        self.apply(subject_effects);
        self.apply(projectile_effects);

        self.react(subject_events);
        self.react(projectile_events);
    }

    fn step_for<F, I, T>(&self, i: usize, all: F) -> (Vec<(I, Vec<Update>)>, Vec<Event<ObjectId>>)
    where
        for<'a> F: Fn(&'a World) -> Vec<(I, &'a T)>,
        I: Copy + Into<ObjectId> + std::fmt::Debug,
        T: Physic + Geo + Region,
    {
        let world = self.ctx.state.world();
        let indexes = self.ctx.state.indexes();
        let all = all(&world);
        let size = (all.len() as f32 / self.ctx.cpus as f32).ceil() as usize;
        if size == 0 {
            return (vec![], vec![]);
        };
        let chunks = all.chunks(size).collect::<Vec<_>>();
        let chunk = chunks.get(i);
        let Some(chunk) = chunk else {
            return (vec![], vec![]);
        };

        // Move code (wich must take worl and indexes as ref because RwReadLockGuard lifetime)
        let objects = |xy| {
            let tile = TileXy(xy);
            let i: WorldTileIndex = tile.into();
            let individuals = indexes.tile_individuals(i);
            // let tile = world.tile(tile);

            let mut objects = vec![];

            for i in individuals {
                let individual = world.individual(*i);
                let individual: Box<&dyn Physic> = Box::new(individual);
                objects.push((ObjectId::Individual(*i), individual));
            }

            if let Some(tile_) = world.tile(tile) {
                let i: WorldTileIndex = tile.into();
                let tile: Box<&dyn Physic> = Box::new(tile_);
                objects.push((ObjectId::Tile(i), tile));
            }

            objects
        };

        let mut events = vec![];
        let updates = chunk
            .into_iter()
            .map(|(i, subject)| {
                let laws = Laws::default();
                let (position, forces, events_) = oc_physics::step(&laws, (i.clone(), *subject), objects);
                tracing::trace!(name="physics-subject", i=?i, position=?position, forces=?forces);
                let updates = changes(i, *subject, &position, &forces);

                tracing::trace!(name = "physics-step-for", i = ?i, updates=?updates, events=events_.len());

                events.extend(events_);
                (*i, updates)
            })
            .collect::<Vec<_>>();

        (updates, events)
    }

    fn write<I, T>(
        &self,
        subjects: Vec<(I, Vec<Update>)>,
    ) -> Vec<(I, (RegionXy, Option<Effect>), Update)>
    where
        I: IntoSubjectMut<T> + std::fmt::Debug + Copy,
        T: Clone + Physic + UpdatePhysic + Geo + UpdateGeo + Region,
    {
        subjects
            .into_iter()
            .flat_map(|(i, updates)| updates.into_iter().map(move |update| (i, update)))
            .filter_map(|(i, update)| {
                let mut world = self.ctx.state.world_mut();
                let Some(subject) = i.into_subject_mut(&mut world) else {
                    return None; // TODO: its possible ? What to do ? Simply log ?
                };
                let effect = write(&update, subject);

                Some((i, effect, update))
            })
            .collect()
    }

    fn apply<'a, I, T: 'a>(&self, effects: Vec<(I, (RegionXy, Option<Effect>), Update)>)
    where
        I: Copy + IntoSubject<T> + IntoNetworkUpdate + IntoIndexEffect<Effect> + std::fmt::Debug,
        T: IntoNetworkInsert<I> + IntoNetworkForgot<I>,
    {
        for (i, (region, effect), update) in effects {
            // Broadcast the update
            let filter = Listening::Regions(vec![region.into()]);
            let messages = vec![i.into_network_update(update)];
            self.ctx.broadcast(filter, messages);

            if let Some(effect) = effect {
                // Update indexes
                {
                    let mut indexes = self.ctx.state.indexes_mut();
                    indexes.react(i.into_index_effect(effect.clone()));
                }

                // Broadcast to new listener if required
                if let Effect::Region { before, after } = effect {
                    let world = self.ctx.state.world();
                    let Some(subject) = i.into_subject(&world) else {
                        continue; // TODO: its possible ? What to do ? Simply log ?
                    };

                    tracing::trace!(name="subject-update-write-broadast-insert", i=?i);
                    let filter = Listening::EnterBorder(before.into(), after.into());
                    let messages = vec![subject.into_network_insert(i)];
                    self.ctx.broadcast(filter, messages);

                    tracing::trace!(name="subject-update-write-broadast-forgot", i=?i);
                    let filter = Listening::ExitBorder(before.into(), after.into());
                    let messages = vec![subject.into_network_forgot(i)];
                    self.ctx.broadcast(filter, messages);
                }
            }
        }
    }

    fn react(&self, events: Vec<Event<ObjectId>>) {
        for event in events {
            match event {
                Event::NoTile(id) => match id {
                    ObjectId::Individual(_) | ObjectId::Tile(_) => {}
                    ObjectId::Projectile(_) => {
                        // FIXME BS NOW: Remove from world
                    }
                },
                Event::Collision(a, b) => match (a, b) {
                    (ObjectId::Individual(_), ObjectId::Individual(_))
                    | (ObjectId::Individual(_), ObjectId::Projectile(_))
                    | (ObjectId::Projectile(_), ObjectId::Projectile(_))
                    | (ObjectId::Individual(_), ObjectId::Tile(_))
                    | (ObjectId::Tile(_), ObjectId::Individual(_))
                    | (ObjectId::Tile(_), ObjectId::Projectile(_))
                    | (ObjectId::Tile(_), ObjectId::Tile(_)) => {}
                    (
                        ObjectId::Projectile(_projectile_id),
                        ObjectId::Individual(_individual_index),
                    ) => {
                        // FIXME: bam
                    }
                    (ObjectId::Projectile(_), ObjectId::Tile(_)) => {
                        tracing::error!("FOOOOOOOOOOOOOOOOOOOOOOOOOOOOOO");
                        // FIXME BS NOW: impact
                    }
                },
            }
        }
    }
}

pub fn changes<I, T>(i: I, subject: &T, position: &[f32; 3], forces: &Vec<Force>) -> Vec<Update>
where
    I: std::fmt::Debug,
    T: Physic + Geo + Region,
{
    tracing::trace!(name="physics-changes", i=?i, position=?position, forces=?forces);
    let mut updates = vec![];

    // TODO: to improve perfs, maybe these updates sould be known at physics::step ?
    if subject.position() != *position {
        updates.push(Update::SetPosition(*position, subject.position()));

        let tile: TileXy = position.clone().into();
        let region: RegionXy = tile.into();

        if subject.tile() != &tile {
            updates.push(Update::SetTile(tile, subject.tile().clone()));
        }

        if subject.region() != &region {
            updates.push(Update::SetRegion(region, subject.region().clone()));
        }
    }

    for force in subject.forces() {
        if !forces.contains(force) {
            updates.push(Update::RemoveForce(force.clone()));
        }
    }

    for force in forces {
        if !subject.forces().contains(force) {
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
        Update::SetPosition(position, _) => {
            subject.set_position(*position);
            None
        }
        Update::SetTile(tile_, _) => {
            subject.set_tile(*tile_);
            Some(Effect::Tile {
                before: tile,
                after: *tile_,
            })
        }
        Update::SetRegion(region_, _) => {
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
        Update::SetVolume(volume, _) => {
            subject.set_volume(volume.clone());
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

impl IntoIndexEffect<Effect> for IndividualIndex {
    fn into_index_effect(&self, effect: Effect) -> index::Effect {
        let effect = index::IndividualEffect::Physic(effect.clone());
        index::Effect::Individual(*self, effect)
    }
}

impl IntoIndexEffect<Effect> for ProjectileId {
    fn into_index_effect(&self, effect: Effect) -> index::Effect {
        let effect = index::ProjectileEffect::Physic(effect.clone());
        index::Effect::Projectile(*self, effect)
    }
}

impl From<IndividualIndex> for ObjectId {
    fn from(value: IndividualIndex) -> Self {
        ObjectId::Individual(value)
    }
}

impl From<ProjectileId> for ObjectId {
    fn from(value: ProjectileId) -> Self {
        ObjectId::Projectile(value)
    }
}
