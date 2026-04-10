use bevy::prelude::*;

use crate::ingame::region::ForgottenRegion;
use crate::world::{InsertTiles, InsertedTiles};

// TODO: gui receive region and tiles, then despawn all, then receive a new time ...
pub fn on_insert_tiles(
    insert: On<InsertTiles>,
    mut world: ResMut<super::World>,
    mut commands: Commands,
) {
    tracing::debug!("Insert tiles for region {:?}", insert.0);
    world.insert_tiles(insert.0, insert.1.clone());
    commands.trigger(InsertedTiles(insert.0));
}

// TODO: should be automatized (macro? derive ?)
pub fn on_forgotten_region(region: On<ForgottenRegion>, mut index: ResMut<super::World>) {
    tracing::debug!("Remove tiles for region {:?}", region.0);
    index.remove_tiles(region.0);
}
