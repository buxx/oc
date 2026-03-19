use bevy::prelude::*;
use oc_geo::{region::WorldRegionIndex, tile::WorldTileIndex};
use oc_root::tile::Tile;
use rustc_hash::FxHashMap;

use crate::ingame::region::ForgottenRegion;
use crate::world::InsertTiles;

// FIXME: use an Index (based on Vec) like server do (check not using too much RAM ...)
// TODO: maybe need improve perf on access tiles. Think about a Vec (with size of world tiles ?) containing references
// or position of tiles in gui vec ?
// #[derive(Debug, Resource, Default, Deref, DerefMut)]
// pub struct Tiles(pub FxHashMap<WorldRegionIndex, FxHashMap<WorldTileIndex, Tile>>);

// impl Tiles {
//     pub fn insert(&mut self, region: WorldRegionIndex, tiles: Vec<(WorldTileIndex, Tile)>) {
//         self.entry(region)
//             .and_modify(|tiles_| {
//                 // TODO: Damn, .clone() is pain here !
//                 for (i, tile) in tiles.clone().into_iter() {
//                     tiles_.insert(i, tile);
//                 }
//             })
//             .or_insert(tiles.into_iter().collect());
//     }

//     // pub fn at(&self, xy: Xy) -> Option<&Tile> {
//     //     let region: WorldRegionIndex = TileXy(xy).into();
//     //     self.get(&region)
//     //         .and_then(|tiles| tiles.get(&TileXy(xy).into()))
//     // }
// }

// TODO: gui receive region and tiles, then despawn all, then receive a new time ...
pub fn on_insert_tiles(insert: On<InsertTiles>, mut index: ResMut<super::World>) {
    tracing::debug!("Insert tiles for region {:?}", insert.0);
    index.insert_tiles(insert.0, insert.1.clone());
}

// TODO: should be automatized (macro? derive ?)
pub fn on_forgotten_region(region: On<ForgottenRegion>, mut index: ResMut<super::World>) {
    tracing::debug!("Remove tiles for region {:?}", region.0);
    index.remove_tiles(region.0);
}
