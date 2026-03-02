use bevy::prelude::*;

#[cfg(feature = "debug")]
use crate::ingame::region::debug;
use crate::{
    ingame::{
        camera::CameraPlugin,
        individual::{IndividualPlugin, on_insert_individual, on_update_individual},
        input::{client::on_to_client, keyboard::on_key_press},
        region::{on_forgotten_region, on_listening_region},
        world::{
            on_despawn_world_map_background, on_spawn_minimap, on_spawn_visible_battle_square,
            on_spawn_world_map_background, on_update_battle_square,
        },
    },
    states::{AppState, Meta},
};
use state::State;

mod camera;
mod draw;
mod individual;
mod init;
mod input;
mod region;
mod state;
mod world;

pub struct IngamePlugin;

impl Plugin for IngamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(IndividualPlugin)
            .add_plugins(CameraPlugin)
            .init_resource::<State>()
            .init_resource::<Meta>()
            .add_observer(on_to_client)
            .add_observer(on_insert_individual)
            .add_observer(on_update_individual)
            .add_observer(on_update_battle_square)
            .add_observer(on_spawn_minimap)
            .add_observer(on_spawn_visible_battle_square)
            .add_observer(on_spawn_world_map_background)
            .add_observer(on_despawn_world_map_background)
            .add_observer(on_listening_region)
            .add_observer(on_forgotten_region)
            // TODO: despawn entities on OnExit(AppState::InGame)
            .add_systems(
                OnEnter(AppState::InGame),
                (init::refresh, init::spawn_world_map),
            )
            .add_systems(Update, on_key_press.run_if(in_state(AppState::InGame)));

        #[cfg(feature = "debug")]
        app.add_observer(debug::on_listening_region)
            .add_observer(debug::on_spawn_region_wire_frame_debug)
            .add_observer(debug::on_forgotten_region)
            .add_observer(debug::on_despawn_region_wire_frame_debug);
    }
}
