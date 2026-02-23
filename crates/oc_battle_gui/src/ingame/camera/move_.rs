use bevy::prelude::*;

pub fn move_(
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
