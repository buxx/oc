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

use crate::{height::Height, meta::Meta, tile::Tile};

pub mod cache;
pub mod control;
pub mod decor;
pub mod flag;
pub mod height;
pub mod interior;
pub mod load;
pub mod map;
pub mod meta;
pub mod physics;
pub mod reader;
pub mod snapshot;
pub mod spawn;
pub mod terrain;
pub mod tile;

#[derive(Constructor)]
pub struct World {
    mod_: Mod, // Maybe its place is not in world (when want to access, need to read lock World, but Mod never change)
    meta: Meta,
    tiles: Vec<Tile>,
    heights: Vec<Height>,
    individuals: Vec<Individual>,
    projectiles: FxHashMap<ProjectileId, Projectile>,
}

macro_rules! region_data {
    ($self:ident, $region:expr, $field:ident, $map:expr) => {{
        let start: TileXy = RegionXy::from($region).into();
        tracing::debug!("Extract region {} {}", $region.0, stringify!($field));
        (0..REGION_HEIGHT)
            .flat_map(move |y| {
                let line_start =
                    WorldTileIndex::from(TileXy(Xy(start.0.0, start.0.1 + y as u64))).0 as usize;
                let line_end = line_start + REGION_WIDTH;
                $self.$field[line_start..line_end]
                    .iter()
                    .zip(line_start..line_end)
                    .map($map)
            })
            .collect()
    }};
}

impl World {
    pub fn tile(&self, xy: TileXy) -> Option<&Tile> {
        self.tiles.get(WorldTileIndex::from(xy).0 as usize)
    }

    // TODO: unit test me
    pub fn region_tiles(&self, region: WorldRegionIndex) -> Vec<(WorldTileIndex, &Tile)> {
        region_data!(self, region, tiles, |(t, i)| (WorldTileIndex(i as u64), t))
    }

    pub fn region_heights(&self, region: WorldRegionIndex) -> Vec<(WorldTileIndex, Height)> {
        region_data!(self, region, heights, |(h, i)| (
            WorldTileIndex(i as u64),
            *h
        ))
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

    use crate::tile::Nature;

    use super::*;

    // TODO: this test is not very good ... To make precise things, we must drop constant usage
    #[test]
    fn test_region_tiles() {
        // Given
        let mod_ = Mod::new("MyMod".to_string(), 1, vec![], vec![], vec![], 1.5);
        let meta = Meta::new("MyWorld".to_string(), 0);
        let tiles: Vec<Tile> = (0..TILES_COUNT)
            .map(|i| Tile::new(WorldTileIndex(i as u64), Nature::ShortGrass, 0))
            .collect();
        let world = World::new(mod_, meta, tiles, vec![], HashMap::default());

        // When
        let tiles = world.region_tiles(WorldRegionIndex(0));
        let tiles: Vec<(WorldTileIndex, Tile)> =
            tiles.into_iter().map(|(i, t)| (i, t.clone())).collect();

        // Then
        let mut expected = vec![];
        for y in 0..REGION_HEIGHT {
            for x in 0..REGION_WIDTH {
                let i: WorldTileIndex = TileXy(Xy(x as u64, y as u64)).into();
                expected.push((i, Tile::new(i, Nature::ShortGrass, 0)));
            }
        }
        assert_eq!(tiles, expected);
    }
}
