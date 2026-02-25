use bevy::prelude::*;

use crate::ingame::camera::{self, map::window_cursor_to_world_map_point, region::UpdateRegions};

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

pub fn move_in_world(
    mut commands: Commands,
    window: Single<&Window>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut state: ResMut<camera::State>,
) {
    if buttons.just_released(MouseButton::Left) {
        let Some(cursor) = window.cursor_position() else {
            return;
        };

        let point = window_cursor_to_world_map_point(cursor, window.size());
        let center = Vec3::new(
            point.x - window.width() / 2.,
            point.y - window.height() / 2.,
            0.,
        );

        tracing::debug!("change battle camera center for {center:?}");
        state.previously = Some(center);

        tracing::debug!("Request update region for {point:?}");
        commands.trigger(UpdateRegions(point));
    }
}
