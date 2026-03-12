use derive_more::Constructor;
use oc_geo::{
    region::{RegionXy, WorldRegionIndex},
    tile::{TileXy, WorldTileIndex},
};
use oc_individual::{Individual, IndividualIndex};
use oc_mod::Mod;
use oc_projectile::{Projectile, ProjectileId};
use oc_root::{REGION_HEIGHT, REGION_WIDTH};
use oc_utils::d2::Xy;
use rustc_hash::FxHashMap;

use crate::{meta::Meta, tile::Tile};

pub mod cache;
pub mod load;
pub mod map;
pub mod meta;
pub mod tile;

#[derive(Constructor)]
pub struct World {
    mod_: Mod, // Maybe its place is not in world (when want to access, need to read lock World, but Mod never change)
    meta: Meta,
    tiles: Vec<tile::Tile>,
    individuals: Vec<Individual>,
    projectiles: FxHashMap<ProjectileId, Projectile>,
}

impl World {
    pub fn tile(&self, xy: TileXy) -> Option<&tile::Tile> {
        self.tiles.get(WorldTileIndex::from(xy).0 as usize)
    }

    // TODO: unit test me
    pub fn region_tiles(&self, region: WorldRegionIndex) -> Vec<(WorldTileIndex, &Tile)> {
        let region: RegionXy = region.into();
        let start: TileXy = region.into();
        let mut tiles = Vec::with_capacity(REGION_WIDTH * REGION_HEIGHT);

        for y in region.0.1 as usize..REGION_HEIGHT {
            let line_start = TileXy(Xy(start.0.0, start.0.1 + y as u64));
            let line_start: WorldTileIndex = line_start.into();
            let line_start = line_start.0 as usize;
            let line_end = line_start + REGION_WIDTH;
            let tiles_ = &self.tiles[line_start..line_end];
            let tiles_: Vec<(WorldTileIndex, &Tile)> = tiles_
                .iter()
                .zip(line_start..line_end)
                .map(|(t, i)| (WorldTileIndex(i as u64), t))
                .collect();
            tiles.extend(tiles_);
        }

        tiles
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

    pub fn mod_(&self) -> &Mod {
        &self.mod_
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use oc_root::TILES_COUNT;

    use super::*;

    #[test]
    fn test_region_tiles() {
        // Given
        let meta = Meta::new("MyWorld".to_string(), 0);
        let tiles = vec![Tile::ShortGrass; TILES_COUNT];
        let world = World::new(meta, tiles, vec![], HashMap::default());

        // When
        let tiles = world.region_tiles(WorldRegionIndex(0));

        // Then
        let mut expected = vec![];
        for y in 0..REGION_HEIGHT {
            for x in 0..REGION_WIDTH {
                let i: WorldTileIndex = TileXy(Xy(x as u64, y as u64)).into();
                expected.push((i, &Tile::ShortGrass));
            }
        }
        assert_eq!(tiles, expected);
    }
}
