use bevy::prelude::*;
use oc_geo::{region::WorldRegionIndex, tile::WorldTileIndex};
use oc_world::tile::Tile;
use rustc_hash::FxHashMap;

use crate::ingame::region::ForgottenRegion;
use crate::world::InsertTiles;

// TODO: maybe need improve perf on access tiles. Think about a Vec (with size of world tiles ?) containing references
// or position of tiles in gui vec ?
#[derive(Debug, Resource, Default, Deref)]
pub struct Tiles(pub FxHashMap<WorldRegionIndex, FxHashMap<WorldTileIndex, Tile>>);

// TODO: gui receive region and tiles, then despawn all, then receive a new time ...
pub fn on_insert_tiles(insert: On<InsertTiles>, mut tiles: ResMut<Tiles>) {
    tracing::debug!("Insert tiles for region {:?}", insert.0);

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

// TODO: should be automatized (macro? derive ?)
pub fn on_forgotten_region(region: On<ForgottenRegion>, mut tiles: ResMut<Tiles>) {
    tracing::debug!("Remove tiles for region {:?}", region.0);

    tiles.0.remove(&region.0);
}
