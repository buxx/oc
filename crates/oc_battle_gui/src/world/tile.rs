use bevy::prelude::*;
use oc_geo::{region::WorldRegionIndex, tile::WorldTileIndex};
use oc_world::tile::Tile;
use rustc_hash::FxHashMap;

use crate::world::InsertTiles;

// TODO: maybe need improve perf on access tiles. Think about a Vec (with size of world tiles ?) containing references
// or position of tiles in gui vec ?
#[derive(Debug, Resource, Default, Deref)]
pub struct Tiles(pub FxHashMap<WorldRegionIndex, FxHashMap<WorldTileIndex, Tile>>);

pub fn on_insert_tiles(insert: On<InsertTiles>, mut tiles: ResMut<Tiles>) {
    tiles
        .0
        .entry(insert.0)
        .and_modify(|tiles| {
            for (i, tile) in &insert.1 {
                tiles.insert(*i, tile.clone());
            }
        })
        .or_insert(insert.1.clone().into_iter().collect());
}
