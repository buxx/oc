use bevy::prelude::*;

use crate::ingame::{
    camera,
    input::map::{SwitchToBattleMap, SwitchToWorldMap},
};

const WORLD_MAP_X: f32 = -100_000.;
const WORLD_MAP_Y: f32 = -100_000.;

pub fn on_switch_to_world_map(
    _: On<SwitchToWorldMap>,
    mut camera: Single<&mut Transform, With<Camera2d>>,
    mut state: ResMut<camera::State>,
) {
    tracing::debug!("Switch to world map");
    let previously = camera.translation;

    state.previously = Some(previously);
    state.focus = camera::Focus::World;
    camera.translation.x = WORLD_MAP_X;
    camera.translation.y = WORLD_MAP_Y;
}

pub fn on_switch_to_battle_map(
    _: On<SwitchToBattleMap>,
    mut camera: Single<&mut Transform, With<Camera2d>>,
    mut state: ResMut<camera::State>,
) {
    tracing::debug!("Switch to battle map");
    let Some(previously) = state.previously else {
        return;
    };

    state.focus = camera::Focus::Battle;
    camera.translation.x = previously.x;
    camera.translation.y = previously.y;
    camera.translation.z = previously.z;
}
