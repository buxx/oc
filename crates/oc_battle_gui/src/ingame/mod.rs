use bevy::prelude::*;

use crate::{
    ingame::{
        individual::IndividualPlugin,
        input::{client::on_to_client, keyboard::on_key_press},
        projectile::ProjectilePlugin,
        region::{on_forgotten_region, on_listening_region},
        world::{
            on_adjust_minimap, on_despawn_world_map_background, on_spawn_minimap,
            on_spawn_visible_battle_square, on_spawn_world_map_background, on_update_battle_square,
        },
    },
    states::AppState,
    world::WorldPlugin,
};
use state::State;

pub mod camera;
mod draw;
mod individual;
mod init;
mod input;
mod physics;
mod projectile;
mod region;
mod state;
mod world;

pub struct IngamePlugin;

#[derive(Debug, Event)]
pub struct FirstIngameEnter;

impl Plugin for IngamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(WorldPlugin)
            .add_plugins(IndividualPlugin)
            .add_plugins(ProjectilePlugin)
            .init_resource::<State>()
            .add_observer(on_to_client)
            .add_observer(on_update_battle_square)
            .add_observer(on_spawn_minimap)
            .add_observer(on_adjust_minimap)
            .add_observer(on_spawn_visible_battle_square)
            .add_observer(on_spawn_world_map_background)
            .add_observer(on_despawn_world_map_background)
            .add_observer(on_listening_region)
            .add_observer(on_forgotten_region)
            // TODO: despawn entities on OnExit(AppState::InGame)
            .add_systems(
                OnEnter(AppState::InGame),
                (init::init, init::refresh, init::spawn_world_map),
            )
            .add_systems(Update, on_key_press.run_if(in_state(AppState::InGame)));

        #[cfg(feature = "debug")]
        app.add_observer(region::debug::on_listening_region)
            .add_observer(region::debug::on_spawn_region_wire_frame_debug)
            .add_observer(region::debug::on_forgotten_region)
            .add_observer(region::debug::on_despawn_region_wire_frame_debug)
            .add_observer(init::on_first_ingame_enter);
    }
}
