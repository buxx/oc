use bevy::prelude::*;
use oc_root::{GEO_PIXELS_PER_TILE, WORLD_HEIGHT, WORLD_WIDTH};

use crate::{
    ingame::{
        camera,
        input::map::{SwitchToBattleMap, SwitchToWorldMap},
    },
    states::InGameState,
};

pub const WORLD_MAP_X: f32 = -100_000.;
pub const WORLD_MAP_Y: f32 = -100_000.;

pub fn on_switch_to_world_map(
    _: On<SwitchToWorldMap>,
    mut camera: Single<&mut Transform, With<Camera2d>>,
    mut state: ResMut<camera::State>,
    window: Single<&Window>,
    mut ingame: ResMut<NextState<InGameState>>,
) {
    tracing::debug!("Switch to world map");
    let previously = camera.translation;

    state.previously = Some(previously);
    state.focus = camera::Focus::World;
    camera.translation.x = WORLD_MAP_X + (window.width() / 2.);
    camera.translation.y = WORLD_MAP_Y + (window.height() / 2.);
    *ingame = NextState::Pending(InGameState::World);
}

pub fn on_switch_to_battle_map(
    _: On<SwitchToBattleMap>,
    mut camera: Single<&mut Transform, With<Camera2d>>,
    mut state: ResMut<camera::State>,
    mut ingame: ResMut<NextState<InGameState>>,
) {
    tracing::debug!("Switch to battle map");
    let Some(previously) = state.previously else {
        return;
    };

    state.focus = camera::Focus::Battle;
    camera.translation.x = previously.x;
    camera.translation.y = previously.y;
    camera.translation.z = previously.z;
    *ingame = NextState::Pending(InGameState::Battle);
}

pub fn window_cursor_to_world_map_point(point: Vec2, window: Vec2) -> Vec2 {
    let cursor = Vec2::new(point.x, window.y - point.y);
    let ratio = cursor / window;
    ratio
        * Vec2::new(
            WORLD_WIDTH as f32 * GEO_PIXELS_PER_TILE as f32,
            WORLD_HEIGHT as f32 * GEO_PIXELS_PER_TILE as f32,
        )
}

pub fn world_map_point_to_bevy_world_point(point: Vec2, window: Vec2) -> Vec2 {
    let ratio = point
        / Vec2::new(
            WORLD_WIDTH as f32 * GEO_PIXELS_PER_TILE as f32,
            WORLD_HEIGHT as f32 * GEO_PIXELS_PER_TILE as f32,
        );
    let point = ratio * window;
    Vec2::new(point.x + WORLD_MAP_X, point.y + WORLD_MAP_Y)
}
