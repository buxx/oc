use derive_more::Constructor;
use oc_geo::{
    region::WorldRegionIndex,
    tile::{TileXy, WorldTileIndex},
};
use oc_individual::{Individual, IndividualIndex};
use oc_projectile::{Projectile, ProjectileId};
use rustc_hash::FxHashMap;

use crate::{meta::Meta, tile::Tile};

pub mod cache;
pub mod load;
pub mod map;
pub mod meta;
pub mod tile;

#[derive(Constructor)]
pub struct World {
    meta: Meta,
    tiles: Vec<tile::Tile>,
    individuals: Vec<Individual>,
    projectiles: FxHashMap<ProjectileId, Projectile>,
}

impl World {
    pub fn tile(&self, xy: TileXy) -> Option<&tile::Tile> {
        self.tiles.get(WorldTileIndex::from(xy).0 as usize)
    }

    pub fn region_tiles(&self, region: WorldRegionIndex) -> Vec<(WorldTileIndex, &Tile)> {
        todo!()
    }

    pub fn individuals(&self) -> &[Individual] {
        &self.individuals
    }

    pub fn individual(&self, i: IndividualIndex) -> &Individual {
        &self.individuals[i.0 as usize]
    }

    pub fn individual_mut(&mut self, i: IndividualIndex) -> &mut Individual {
        &mut self.individuals[i.0 as usize]
    }

    pub fn projectiles(&self) -> Vec<(&ProjectileId, &Projectile)> {
        self.projectiles.iter().collect()
    }

    pub fn projectiles_mut(&mut self) -> &mut FxHashMap<ProjectileId, Projectile> {
        &mut self.projectiles
    }

    pub fn projectile(&self, i: &ProjectileId) -> Option<&Projectile> {
        self.projectiles.get(i)
    }

    pub fn projectile_mut(&mut self, i: &ProjectileId) -> Option<&mut Projectile> {
        self.projectiles.get_mut(i)
    }

    pub fn meta(&self) -> &Meta {
        &self.meta
    }
}
