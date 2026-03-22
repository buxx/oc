use std::f32;

use oc_geo::{
    region::WorldRegionIndex,
    tile::{TileXy, WorldTileIndex},
};
use oc_physics::{
    Force, Physic,
    collision::{Material, Materials},
    volume::Volume,
};
use oc_root::GEO_PIXELS_PER_TILE;

use crate::World;
use derive_more::Constructor;
use rkyv::{Archive, Deserialize, Serialize};

#[derive(Debug, Clone, Archive, Deserialize, Serialize, PartialEq, Constructor)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct Tile {
    pub i: WorldTileIndex, // Should not be necessary, but oc_physics::step must take a reference ...
    pub nature: Nature,
    pub z: u8,
}

#[derive(Debug, Clone, Archive, Deserialize, Serialize, PartialEq)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub enum Nature {
    ShortGrass,
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
    fn into_tiles<'a>(&self, world: &'a World) -> Vec<(WorldTileIndex, Tile)>;
}

impl IntoTiles for WorldRegionIndex {
    fn into_tiles<'a>(&self, world: &'a World) -> Vec<(WorldTileIndex, Tile)> {
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
    fn position(&self) -> [f32; 3] {
        let xy: TileXy = self.i.into();
        let point = xy.point();
        [point[0], point[1], self.z as f32]
    }

    fn forces(&self) -> &Vec<Force> {
        static EMPTY: Vec<Force> = vec![];
        &EMPTY
    }

    fn volume(&self) -> &Volume {
        // FIXME: according to tile type ...
        &Volume::Cube {
            width: GEO_PIXELS_PER_TILE as f32,
            height: GEO_PIXELS_PER_TILE as f32,
            depth: f32::MIN,
        }
    }
}
