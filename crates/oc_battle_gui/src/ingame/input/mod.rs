use bevy::prelude::*;

use crate::states::AppState;

pub mod client;
pub mod individual;
pub mod keyboard;
pub mod left_click;
pub mod projectile;

#[derive(Debug, Resource, Default)]
pub struct State {
    #[cfg(feature = "debug")]
    pub clicks: Vec<Vec2>,
}

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<State>()
            .add_observer(client::on_to_client)
            .add_systems(
                Update,
                (keyboard::on_key_press,).run_if(in_state(AppState::InGame)),
            )
            .add_systems(
                Update,
                (left_click::show).run_if(in_state(AppState::InGame)),
            );

        #[cfg(feature = "debug")]
        app.init_resource::<left_click::SpawnProjectileLeftClick>()
            .init_resource::<left_click::LeftClick>()
            .add_systems(
                Update,
                (left_click::update_spawn_projectile_clicks_line,)
                    .run_if(in_state(AppState::InGame)),
            )
            .add_observer(left_click::on_set_left_click)
            .add_observer(left_click::on_set_spawn_projectile_left_click)
            .add_observer(left_click::on_spawn_clicks_line)
            .add_observer(left_click::on_despawn_clicks_line);
    }
}
