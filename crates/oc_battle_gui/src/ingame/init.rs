use bevy::prelude::*;
use oc_network::ToServer;

use crate::{
    ingame::world::{SpawnMinimap, SpawnVisibleBattleSquare, SpawnWorldMapBackground},
    network::output::ToServerEvent,
};

pub fn refresh(mut commands: Commands) {
    commands.trigger(ToServerEvent(ToServer::Refresh.into()));
}

pub fn spawn_world_map(mut commands: Commands) {
    commands.trigger(SpawnMinimap);
    commands.trigger(SpawnWorldMapBackground);
    commands.trigger(SpawnVisibleBattleSquare);
}
