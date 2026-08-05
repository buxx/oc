use bevy::prelude::*;
use oc_root::geo::{ScreenVec2, WorldVec2};
use oc_root::{Wcfg, WcfgFrom, WorldConfig};
use oc_utils::{let_ok, let_some, return_if};

use crate::ingame;
use crate::ingame::input::left_click::{LeftClick, LeftClickMode, SetLeftClick};
use crate::ingame::lov::{DespawnLov, LovClickMode, SpawnLov, SpawnLovConfig, SpawnLovProfile};

pub fn system(
    mut commands: Commands,
    w: Res<Wcfg>,
    window: Single<&Window>,
    camera: Single<(&Camera, &GlobalTransform)>,
    buttons: Res<ButtonInput<MouseButton>>,
    mode: Res<LeftClick>,
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<crate::ingame::input::State>,
) {
    let_some!(w = &w.0, return);
    let_some!(cursor = window.cursor_position(), return);
    let (camera, transform) = *camera;
    let point = camera.viewport_to_world_2d(transform, cursor);
    let_ok!(point = point, return);
    let point = ScreenVec2::new(point.x, point.y);
    let point = WorldVec2::from_(point, &w);

    let LeftClickMode::LineOfView(profile) = &mode.0 else {
        return;
    };

    return_if!(maybe_cancel(&mut commands, &buttons, &keys));
    show(w, point, &mut commands, &buttons, &mut state, &profile);
}

pub fn show(
    _w: &WorldConfig,
    point: WorldVec2,
    commands: &mut Commands,
    buttons: &ButtonInput<MouseButton>,
    state: &mut ingame::input::State,
    profile: &SpawnLovConfig,
) {
    match profile.click {
        LovClickMode::DraggedClick => {
            if buttons.just_pressed(MouseButton::Left) {
                tracing::trace!(name = "ingame-input-left-click-lov-dragged-pressed");
                state.clicks.push(point);
                commands.trigger(SpawnLov(SpawnLovProfile {
                    start: point,
                    start_plus_z: profile.start_plus_z,
                    stop_plus_z: profile.stop_plus_z,
                }));
            }

            if buttons.just_released(MouseButton::Left) {
                tracing::trace!(name = "ingame-input-left-click-lov-dragged-release");
                state.clicks.clear();
                commands.trigger(DespawnLov);
            }
        }
        LovClickMode::TwoClicks => {
            todo!()
        }
    }
}

fn maybe_cancel(
    commands: &mut Commands,
    buttons: &ButtonInput<MouseButton>,
    keys: &ButtonInput<KeyCode>,
) -> bool {
    if keys.just_pressed(KeyCode::Escape) || buttons.just_pressed(MouseButton::Middle) {
        // TODO: need despawn things about lov display
        commands.trigger(SetLeftClick(LeftClickMode::Select));
        return true;
    }
    false
}
