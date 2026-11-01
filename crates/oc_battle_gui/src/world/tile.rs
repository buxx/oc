use bevy::prelude::*;
use oc_utils::let_some;

use crate::ingame::region::ForgottenRegion;
use crate::states::GameConfig;
use crate::world::{InsertAlteredTiles, InsertTiles, InsertedTiles};

// TODO: gui receive region and tiles, then despawn all, then receive a new time ...
pub fn on_insert_tiles(
    insert: On<InsertTiles>,
    g: Res<GameConfig>,
    mut world: ResMut<super::World>,
    mut commands: Commands,
) {
    tracing::debug!("Insert tiles for region {:?}", insert.0);
    world.insert_tiles(insert.0, insert.1.clone());
    let_some!(g_ = &g.0, return);

    // Alter tile must apply on altered tiles
    world.apply_altered_tiles(insert.0, &g_.mod_);
    commands.trigger(InsertedTiles(insert.0));

    // FIXME: maybe its too much cpu blocking. Delay it ? Async it ?
    world.update_navmesh(&g_.w, &g_.mod_);
}

pub fn on_insert_altered_tiles(
    insert: On<InsertAlteredTiles>,
    g: Res<GameConfig>,
    mut world: ResMut<super::World>,
) {
    tracing::debug!("Insert altered tiles for region {:?}", insert.0);
    let_some!(g = &g.0, return);
    world.insert_altered_tiles(insert.0, insert.1.clone(), &g.mod_);
    // TODO: maybe perormance problem ? (ui freeze)
    world.update_navmesh(&g.w, &g.mod_);
}

// TODO: should be automatized (macro? derive ?)
pub fn on_forgotten_region(region: On<ForgottenRegion>, mut index: ResMut<super::World>) {
    tracing::debug!("Remove tiles for region {:?}", region.0);
    index.remove_tiles(region.0);
}
