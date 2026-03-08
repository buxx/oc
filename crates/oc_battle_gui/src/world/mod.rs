use bevy::prelude::*;
use oc_geo::{region::WorldRegionIndex, tile::WorldTileIndex};
use oc_world::tile::Tile;

use crate::world::tile::on_insert_tiles;

pub mod tile;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<tile::Tiles>()
            .add_observer(on_insert_tiles);
    }
}

#[derive(Debug, Event)]
pub struct InsertTiles(pub WorldRegionIndex, pub Vec<(WorldTileIndex, Tile)>);
