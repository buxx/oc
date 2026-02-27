use bevy::prelude::*;
use oc_root::{WORLD_HEIGHT_PIXELS, WORLD_WIDTH_PIXELS};

use crate::{
    ingame::{
        camera,
        draw::world::WorldMapDisplay,
        input::map::{SwitchToBattleMap, SwitchToWorldMap},
    },
    states::InGameState,
};

#[derive(Debug, Event)]
pub struct SaveCurrentWindowCenterAsBattleCenter;

pub fn on_switch_to_world_map(
    _: On<SwitchToWorldMap>,
    mut camera: Single<&mut Transform, With<Camera2d>>,
    mut state: ResMut<camera::State>,
    window: Single<&Window>,
    mut ingame: ResMut<NextState<InGameState>>,
) {
    tracing::debug!("Switch to world map");
    let display = WorldMapDisplay::from_env(window.size());
    state.focus = camera::Focus::World;
    camera.translation.x = display.center.x;
    camera.translation.y = display.center.y;
    *ingame = NextState::Pending(InGameState::World);
}

pub fn on_save_current_window_center_as_battle_center(
    _: On<SaveCurrentWindowCenterAsBattleCenter>,
    camera: Single<&mut Transform, With<Camera2d>>,
    mut state: ResMut<camera::State>,
) {
    let point = camera.translation;
    tracing::debug!("Save {point:?} as battle center");
    state.previously = Some(point);
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

pub fn window_point_to_world_map_point(point: Vec2, window: Vec2) -> Vec2 {
    let display = WorldMapDisplay::from_env(window);
    let point = Vec2::new(point.x, display.size.y - point.y); // Invert y (as bevy y way)
    let point = point - display.padding;
    let ratio = point / display.size;
    ratio * Vec2::new(WORLD_WIDTH_PIXELS as f32, WORLD_HEIGHT_PIXELS as f32)
}

pub fn world_map_point_to_bevy_world_point(point: Vec2, window: Vec2) -> Vec2 {
    let display = WorldMapDisplay::from_env(window);
    let ratio = point / Vec2::new(WORLD_WIDTH_PIXELS as f32, WORLD_HEIGHT_PIXELS as f32);
    let point = ratio * display.size;
    Vec2::new(point.x + display.start.x, point.y + display.start.y)
}
