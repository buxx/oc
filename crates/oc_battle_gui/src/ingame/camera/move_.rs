use bevy::prelude::*;
use oc_root::Wcfg;
use oc_utils::{let_ok, let_some};

use crate::ingame::camera::{
    self, GoToPoint, map::window_point_to_world_map_point, region::UpdateRegions,
};

#[derive(Debug, Event, Deref)]
pub struct CenterCameraOn(pub Vec2);

#[derive(Debug, Event)]
pub struct MovedBattleCamera;

#[derive(Debug, Event)]
pub struct UpdateVisibleBattleSquare(pub Vec2); // The bevy world map point correspnding to the center of the screen

pub fn move_battle(
    mut commands: Commands,
    mut camera: Single<&mut Transform, With<Camera2d>>,
    window: Single<&Window>,
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    state: Res<super::State>,
) {
    let cursor = window.cursor_position();
    let ctrl = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    let left = buttons.pressed(MouseButton::Left);
    let right = buttons.pressed(MouseButton::Right);

    if (right || (left && ctrl))
        && let (Some(cursor1), Some(cursor2)) = (&state.cursor, &cursor)
    {
        let diff = cursor1 - cursor2;
        if diff != Vec2::ZERO {
            camera.translation.x += diff.x;
            camera.translation.y -= diff.y;
            tracing::trace!(name = "ingame-camera-trigger-moved-battle-camera");
            commands.trigger(MovedBattleCamera)
        }
    }
}

pub fn on_center_camera_on(
    point: On<CenterCameraOn>,
    mut commands: Commands,
    mut camera: Single<&mut Transform, With<Camera2d>>,
) {
    camera.translation.x = point.x;
    camera.translation.y = point.y;
    commands.trigger(MovedBattleCamera)
}

pub fn on_moved_battle_camera(
    _: On<MovedBattleCamera>,
    mut commands: Commands,
    camera: Single<(&Camera, &GlobalTransform)>,
    window: Single<&Window>,
) {
    let (camera, transform) = *camera;
    let width = window.resolution.width();
    let height = window.resolution.height();
    let center = Vec2::new(width / 2., height / 2.);
    let center = camera.viewport_to_world_2d(transform, center);
    let_ok!(center = center, return);

    commands.trigger(UpdateVisibleBattleSquare(center));
    commands.trigger(UpdateRegions(center));
}

pub fn move_in_world_map(
    mut commands: Commands,
    w: Res<Wcfg>,
    window: Single<&Window>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut state: ResMut<camera::State>,
) {
    let_some!(w = &w.0, return);

    if buttons.just_released(MouseButton::Left) {
        let_some!(cursor = window.cursor_position(), return);

        let point = window_point_to_world_map_point(w, cursor, window.size());
        let center = Vec3::new(
            point.x - window.width() / 2.,
            point.y - window.height() / 2.,
            0.,
        );

        tracing::debug!("change battle camera center for {center:?}");
        state.previously = Some(Vec3::new(point.x, point.y, 0.0));

        tracing::debug!("Request update region for {point:?}");
        commands.trigger(UpdateRegions(point));
        commands.trigger(UpdateVisibleBattleSquare(point));
    }
}

pub fn on_go_to_point(
    point: On<GoToPoint>,
    mut commands: Commands,
    mut camera: Single<&mut Transform, With<Camera2d>>,
) {
    tracing::debug!("Moved on {point:?}");
    camera.translation.x = point.x;
    camera.translation.y = point.y;
    camera.translation.z = 0.;
    commands.trigger(UpdateRegions(point.0.into()));
    commands.trigger(UpdateVisibleBattleSquare(point.0.into()));
}
