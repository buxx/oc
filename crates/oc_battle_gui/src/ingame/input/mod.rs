use bevy::prelude::*;
use oc_root::geo::WorldVec2;

use crate::{
    ingame::{InGameState, input::left_click::LeftClickModeType},
    states::AppState,
};

pub mod client;
pub mod individual;
pub mod keyboard;
pub mod left_click;
pub mod projectile;

#[derive(Debug, Resource, Default)]
pub struct State {
    pub clicks: Vec<WorldVec2>,
}

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<LeftClickModeType>()
            .init_resource::<State>()
            .init_resource::<left_click::order::OnGoing>()
            .init_resource::<left_click::LeftClick>()
            .add_observer(client::on_to_client)
            .add_observer(left_click::on_set_left_click)
            .add_observer(left_click::on_spawn_clicks_line)
            .add_observer(left_click::on_despawn_clicks_line)
            .add_systems(
                Update,
                (keyboard::on_key_press,).run_if(in_state(AppState::InGame)),
            )
            .add_systems(
                Update,
                (left_click::order::system)
                    .run_if(in_state(AppState::InGame))
                    .run_if(in_state(InGameState::Battle))
                    .run_if(in_state(LeftClickModeType::Order)),
            )
            .add_systems(
                Update,
                (left_click::lov::system)
                    .run_if(in_state(AppState::InGame))
                    .run_if(in_state(InGameState::Battle))
                    .run_if(in_state(LeftClickModeType::LineOfView)),
            );

        #[cfg(feature = "debug")]
        app.init_resource::<left_click::SpawnProjectileLeftClick>()
            .add_systems(
                Update,
                (left_click::update_spawn_projectile_clicks_line,)
                    .run_if(in_state(AppState::InGame)),
            )
            .add_observer(left_click::on_set_spawn_projectile_left_click)
            .add_systems(
                Update,
                (left_click::spawn_projectile::system)
                    .run_if(in_state(AppState::InGame))
                    .run_if(in_state(InGameState::Battle))
                    .run_if(in_state(LeftClickModeType::SpawnProjectile)),
            );
    }
}
