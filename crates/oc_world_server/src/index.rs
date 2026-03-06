use std::ops::{Deref, DerefMut};

use oc_geo::{
    region::{Region, RegionXy, WorldRegionIndex},
    tile::{TileXy, WorldTileIndex},
};
use oc_individual::IndividualIndex;
use oc_projectile::{Projectile, ProjectileId};
use oc_root::{REGIONS_COUNT, TILES_COUNT};
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
        let mut tiles_individuals = SizedIndex::new(TILES_COUNT);
        let mut tiles_projectiles = SizedIndex::new(TILES_COUNT);
        let mut regions_individuals = SizedIndex::new(REGIONS_COUNT);
        let mut regions_projectiles = SizedIndex::new(REGIONS_COUNT);

        for (i, individual) in world.individuals().iter().enumerate() {
            let tile: WorldTileIndex = individual.tile.into();
            let region: WorldRegionIndex = tile.into();

            tiles_individuals[tile.0].push(i.into());
            regions_individuals[region.0 as usize].push(i.into());
        }

        for (id, projectile) in world.projectiles() {
            let tile: WorldTileIndex = projectile.tile().clone().into();
            let region: WorldRegionIndex = tile.into();

            tiles_projectiles[tile.0].push(*id);
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
        self.update_projectile_tile(id, projectile.tile().clone(), projectile.tile().clone());
        self.update_projectile_region(id, projectile.region().clone(), projectile.region().clone());
    }

    fn update_individual_tile(&mut self, i: IndividualIndex, now: TileXy, before: TileXy) {
        let before_tile: WorldTileIndex = before.into();
        self.tiles_individuals[before_tile.0].retain(|i_| *i_ != i);
        let now_tile: WorldTileIndex = now.into();
        self.tiles_individuals[now_tile.0].push(i);
    }

    fn update_projectile_tile(&mut self, id: ProjectileId, now: TileXy, before: TileXy) {
        let before_tile: WorldTileIndex = before.into();
        self.tiles_projectiles[before_tile.0].retain(|id_| *id_ != id);
        let now_tile: WorldTileIndex = now.into();
        self.tiles_projectiles[now_tile.0].push(id);
    }

    fn update_individual_region(&mut self, i: IndividualIndex, now: RegionXy, before: RegionXy) {
        let before_region: WorldRegionIndex = before.into();
        self.regions_individuals[before_region.0 as usize].retain(|i_| *i_ != i);
        let now_region: WorldRegionIndex = now.into();
        self.regions_individuals[now_region.0 as usize].push(i);
    }

    fn update_projectile_region(&mut self, id: ProjectileId, now: RegionXy, before: RegionXy) {
        let before_region: WorldRegionIndex = before.into();
        self.regions_projectiles[before_region.0 as usize].retain(|id_| *id_ != id);
        let now_region: WorldRegionIndex = now.into();
        self.regions_projectiles[now_region.0 as usize].push(id);
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
