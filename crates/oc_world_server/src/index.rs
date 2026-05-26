use std::ops::{Deref, DerefMut};

use oc_geo::region::Region;
use oc_geo::{region::WorldRegionIndex, tile::WorldTileIndex};
use oc_individual::IndividualIndex;
use oc_projectile::Projectile;
use oc_projectile::ProjectileId;
use oc_root::WcfgInto;
use oc_world::World;

use crate::physics;

pub struct SizedIndex<T>(Vec<Vec<T>>);

impl<T: std::clone::Clone> SizedIndex<T> {
    pub fn new(size: usize) -> Self {
        Self(vec![vec![]; size])
    }
}

impl<T> Deref for SizedIndex<T> {
    type Target = Vec<Vec<T>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for SizedIndex<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub struct Indexes {
    tiles_individuals: SizedIndex<IndividualIndex>,
    tiles_projectiles: SizedIndex<ProjectileId>,
    regions_individuals: SizedIndex<IndividualIndex>,
    regions_projectiles: SizedIndex<ProjectileId>,
}

impl Indexes {
    pub fn new(world: &World) -> Self {
        let mut tiles_individuals = SizedIndex::new(world.w.tiles_count as usize);
        let mut tiles_projectiles = SizedIndex::new(world.w.tiles_count as usize);
        let mut regions_individuals = SizedIndex::new(world.w.regions_count as usize);
        let mut regions_projectiles = SizedIndex::new(world.w.regions_count as usize);

        for (i, individual) in world.individuals().iter().enumerate() {
            let tile: WorldTileIndex = individual.tile;
            let region: WorldRegionIndex = tile.into_(&world.w);

            tiles_individuals[tile.0 as usize].push(i.into());
            regions_individuals[region.0 as usize].push(i.into());
        }

        for (id, projectile) in world.projectiles() {
            let tile: WorldTileIndex = projectile.tile();
            let region: WorldRegionIndex = tile.into_(&world.w);

            tiles_projectiles[tile.0 as usize].push(*id);
            regions_projectiles[region.0 as usize].push(*id);
        }

        Self {
            tiles_individuals,
            tiles_projectiles,
            regions_individuals,
            regions_projectiles,
        }
    }

    pub fn insert_projectile(&mut self, id: ProjectileId, projectile: &Projectile) {
        self.update_projectile_tile(id, projectile.tile(), projectile.tile());
        self.update_projectile_region(id, projectile.region(), projectile.region());
    }

    pub fn remove_projectile(&mut self, id: &ProjectileId, projectile: &Projectile) {
        let tile = projectile.tile();
        self.tiles_projectiles[tile.0 as usize].retain(|p| p != id);

        let region = projectile.region();
        self.regions_projectiles[region.0 as usize].retain(|p| p != id);
    }

    fn update_individual_tile(
        &mut self,
        i: IndividualIndex,
        now: WorldTileIndex,
        before: WorldTileIndex,
    ) {
        self.tiles_individuals[before.0 as usize].retain(|i_| *i_ != i);
        self.tiles_individuals[now.0 as usize].push(i);
    }

    fn update_projectile_tile(
        &mut self,
        id: ProjectileId,
        now: WorldTileIndex,
        before: WorldTileIndex,
    ) {
        self.tiles_projectiles[before.0 as usize].retain(|id_| *id_ != id);
        self.tiles_projectiles[now.0 as usize].push(id);
    }

    fn update_individual_region(
        &mut self,
        i: IndividualIndex,
        now: WorldRegionIndex,
        before: WorldRegionIndex,
    ) {
        self.regions_individuals[before.0 as usize].retain(|i_| *i_ != i);
        self.regions_individuals[now.0 as usize].push(i);
    }

    fn update_projectile_region(
        &mut self,
        id: ProjectileId,
        now: WorldRegionIndex,
        before: WorldRegionIndex,
    ) {
        self.regions_projectiles[before.0 as usize].retain(|id_| *id_ != id);
        self.regions_projectiles[now.0 as usize].push(id);
    }

    pub fn tile_individuals(&self, tile: WorldTileIndex) -> &Vec<IndividualIndex> {
        &self.tiles_individuals[tile.0 as usize]
    }

    pub fn region_individuals(&self, region: WorldRegionIndex) -> &Vec<IndividualIndex> {
        &self.regions_individuals[region.0 as usize]
    }

    pub fn region_projectiles(&self, region: WorldRegionIndex) -> &Vec<ProjectileId> {
        &self.regions_projectiles[region.0 as usize]
    }

    pub fn react(&mut self, effect: Effect) {
        match effect {
            Effect::Individual(i, effect) => match effect {
                IndividualEffect::Physic(effect) => match effect {
                    physics::Effect::Tile { before, after } => {
                        self.update_individual_tile(i, after, before)
                    }
                    physics::Effect::Region { before, after } => {
                        self.update_individual_region(i, after, before)
                    }
                },
            },
            Effect::Projectile(i, effect) => match effect {
                ProjectileEffect::Physic(effect) => match effect {
                    physics::Effect::Tile { before, after } => {
                        self.update_projectile_tile(i, before, after)
                    }
                    physics::Effect::Region { before, after } => {
                        self.update_projectile_region(i, after, before)
                    }
                },
            },
        }
    }
}

pub enum Effect {
    Individual(IndividualIndex, IndividualEffect),
    Projectile(ProjectileId, ProjectileEffect),
}

pub enum IndividualEffect {
    Physic(physics::Effect),
}

pub enum ProjectileEffect {
    Physic(physics::Effect),
}

pub trait IntoIndexEffect<T> {
    fn into_index_effect(&self, value: T) -> Effect;
}
