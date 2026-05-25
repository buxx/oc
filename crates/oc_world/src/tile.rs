use std::f32;

use oc_geo::{
    region::WorldRegionIndex,
    tile::{TileXy, WorldTileIndex},
};
use oc_mod::{Mod, nature::NatureIndex};
use oc_physics::{
    Force, Physic,
    collision::{Material, Materials},
    volume::Volume,
};
use oc_root::{WcfgInto, WorldConfig};

use crate::World;
use derive_more::Constructor;
use rkyv::{Archive, Deserialize, Serialize};

const DEPTH: f32 = 10_000.;

#[derive(Debug, Clone, Archive, Deserialize, Serialize, PartialEq, Constructor)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct Tile {
    pub i: WorldTileIndex, // Should not be necessary, but oc_physics::step must take a reference ...
    pub nature: NatureIndex,
    pub z: u8,
}

impl Tile {
    pub fn z_pixels(&self, w: &WorldConfig) -> f32 {
        self.z as f32 * w.geo_meters_per_z.0 * w.geo_pixels_per_meters
    }
}

pub trait AsTiles {
    fn as_tiles<'a>(&self, world: &'a World) -> Vec<(WorldTileIndex, &'a Tile)>;
}

impl AsTiles for WorldRegionIndex {
    fn as_tiles<'a>(&self, world: &'a World) -> Vec<(WorldTileIndex, &'a Tile)> {
        world.region_tiles(*self)
    }
}

pub trait IntoTiles {
    fn into_tiles(&self, world: &World) -> Vec<(WorldTileIndex, Tile)>;
}

impl IntoTiles for WorldRegionIndex {
    fn into_tiles(&self, world: &World) -> Vec<(WorldTileIndex, Tile)> {
        let tiles = self.as_tiles(world);
        tiles.into_iter().map(|(i, t)| (i, t.clone())).collect()
    }
}

impl Material for Tile {
    fn material(&self) -> Materials {
        // TODO: depending on tile
        Materials::Traversable
    }
}

impl Physic for Tile {
    fn position(&self, w: &WorldConfig) -> [f32; 3] {
        let xy: TileXy = self.i.into_(w);
        let point = xy.point(w);
        [
            point[0],
            point[1],
            self.z as f32 * w.geo_meters_per_z.0 * w.geo_pixels_per_meters,
        ]
    }

    fn forces(&self, _: &WorldConfig) -> &Vec<Force> {
        static EMPTY: Vec<Force> = vec![];
        &EMPTY
    }

    fn volume(&self, ref_: [f32; 3], w: &WorldConfig, mod_: &Mod) -> Volume {
        tracing::trace!(name = "tile-volume", ref_ = ?ref_);
        let nature = mod_.nature(self.nature);
        let exceedance = nature.z.0 * w.geo_pixels_per_meters;
        Volume::Cube {
            x: ref_[0],
            y: ref_[1],
            z: -DEPTH,
            width: w.geo_pixels_per_tile as f32,
            height: w.geo_pixels_per_tile as f32,
            depth: DEPTH + ref_[2] + exceedance,
        }
    }
}
