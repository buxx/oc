use bevy::prelude::*;
use oc_root::{GEO_PIXELS_PER_TILE, WORLD_HEIGHT, WORLD_WIDTH};

use crate::ingame::{camera::map::window_cursor_to_world_map_point, draw};

pub fn move_battle(
    mut camera: Single<&mut Transform, With<Camera2d>>,
    window: Single<&Window>,
    buttons: Res<ButtonInput<MouseButton>>,
    state: Res<super::State>,
) {
    let cursor = window.cursor_position();

    if buttons.pressed(MouseButton::Left) {
        if let (Some(cursor1), Some(cursor2)) = (&state.cursor, &cursor) {
            let diff = cursor1 - cursor2;
            camera.translation.x += diff.x;
            camera.translation.y -= diff.y;
        }
    }
}

// TODO: in debug, display cursor position on the world
pub fn move_in_world(
    mut camera: Single<&mut Transform, With<Camera2d>>,
    window: Single<&Window>,
    buttons: Res<ButtonInput<MouseButton>>,
) {
    if buttons.just_released(MouseButton::Left) {
        let Some(cursor) = window.cursor_position() else {
            return;
        };

        let point = window_cursor_to_world_map_point(cursor, window.size());
        dbg!(point);
    }
}
