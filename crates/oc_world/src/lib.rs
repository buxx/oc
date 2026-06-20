use derive_more::Constructor;
use oc_geo::{
    region::{RegionXy, WorldRegionIndex},
    tile::{TileXy, WorldTileIndex},
};
use oc_individual::{
    Individual, IndividualIndex,
    squad::{Squad, SquadIndex},
};
use oc_mod::Mod;
use oc_projectile::{Projectile, ProjectileId};
use oc_root::{WcfgInto, WorldConfig, identity::Identity};
use oc_utils::d2::Xy;
use rustc_hash::FxHashMap;

use crate::{meta::Meta, resume::WorldResume, tile::Tile};

pub mod cache;
pub mod control;
pub mod decor;
pub mod flag;
pub mod glb;
pub mod interior;
pub mod load;
pub mod map;
pub mod meta;
pub mod navmesh;
pub mod physics;
pub mod reader;
pub mod resume;
pub mod snapshot;
pub mod spawn;
pub mod terrain;
pub mod tile;

#[derive(Constructor)]
pub struct World {
    pub w: WorldConfig,
    pub mod_: Mod, // Maybe its place is not in world (when want to access, need to read lock World, but Mod never change)
    pub meta: Meta,
    pub tiles: Vec<Tile>,
    pub navmesh: polyanya::Mesh,
    pub individuals: Vec<Individual>,
    pub squads: Vec<Squad>,
    pub projectiles: FxHashMap<ProjectileId, Projectile>,
}

impl World {
    pub fn tile(&self, i: WorldTileIndex) -> Option<&Tile> {
        self.tiles.get(i.0 as usize)
    }

    // TODO: unit test me
    pub fn region_tiles(&self, region: WorldRegionIndex) -> Vec<(WorldTileIndex, &Tile)> {
        let region_: RegionXy = region.into_(&self.w);
        let start: TileXy = region_.into_(&self.w);
        let count = self.w.region_width * self.w.region_height;
        let mut tiles = Vec::with_capacity(count as usize);

        tracing::debug!("Extract region {} tiles", region.0,);
        for y in 0..self.w.region_height {
            let line_start = TileXy(Xy(start.0.0, start.0.1 + y));
            let line_start: WorldTileIndex = line_start.into_(&self.w);
            let line_start = line_start.0;
            let line_end = line_start + self.w.region_width;
            let tiles_ = &self.tiles[line_start as usize..line_end as usize];
            let tiles_: Vec<(WorldTileIndex, &Tile)> = tiles_
                .iter()
                .zip(line_start..line_end)
                .map(|(t, i)| (WorldTileIndex(i), t))
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

    pub fn squad(&self, i: SquadIndex) -> &Squad {
        &self.squads[i.0 as usize]
    }

    pub fn squad_mut(&mut self, i: SquadIndex) -> &mut Squad {
        &mut self.squads[i.0 as usize]
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

    pub fn resume(&self, identity: &Identity) -> WorldResume {
        let squads = self
            .squads
            .iter()
            .enumerate()
            .filter(|(_, s)| s.side == identity.side)
            .map(|(i, s)| (SquadIndex(i as u64), s.clone()))
            .collect();

        WorldResume { squads }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use oc_mod::nature::{NatureIndex, Traversability};
    use oc_root::physics::Meters;

    use super::*;

    // TODO: this test is not very good ... To make precise things, we must drop constant usage
    #[test]
    fn test_region_tiles() {
        // Given
        let w = WorldConfig::new(1000, 1000, Meters(0.1));
        let mod_ = Mod::new("MyMod".to_string(), 1, vec![], vec![], vec![], vec![], 1.5);
        let meta = Meta::new("MyWorld".to_string(), 0, w.geo_meters_per_z.0);
        let tiles: Vec<Tile> = (0..w.tiles_count)
            .map(|i| {
                Tile::new(
                    WorldTileIndex(i as u64),
                    NatureIndex(0),
                    0,
                    Traversability::all(),
                )
            })
            .collect();
        let world = World::new(
            w.clone(),
            mod_,
            meta,
            tiles,
            polyanya::Mesh::default(),
            vec![],
            vec![],
            HashMap::default(),
        );

        // When
        let tiles = world.region_tiles(WorldRegionIndex(0));
        let tiles: Vec<(WorldTileIndex, Tile)> =
            tiles.into_iter().map(|(i, t)| (i, t.clone())).collect();

        // Then
        let mut expected = vec![];
        for y in 0..w.region_height as usize {
            for x in 0..w.region_width as usize {
                let i: WorldTileIndex = TileXy(Xy(x as u64, y as u64)).into_(&w);
                expected.push((i, Tile::new(i, NatureIndex(0), 0, Traversability::all())));
            }
        }
        assert_eq!(tiles, expected);
    }
}
