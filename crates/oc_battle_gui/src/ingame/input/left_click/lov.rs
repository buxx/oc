use bevy::prelude::*;
use oc_root::{Wcfg, WorldConfig};
use oc_utils::{let_ok, let_some, return_if};

use crate::ingame;
use crate::ingame::input::left_click::{LeftClick, LeftClickMode, SetLeftClick};
use crate::ingame::lov::{DespawnLov, LovClickMode, SpawnLov, SpawnLovConfig, SpawnLovProfile};
use crate::window::PointerInWindow;

pub fn system(
    mut commands: Commands,
    w: Res<Wcfg>,
    ignore: Res<PointerInWindow>, // TODO: use state ?
    window: Single<&Window>,
    camera: Single<(&Camera, &GlobalTransform)>,
    buttons: Res<ButtonInput<MouseButton>>,
    mode: Res<LeftClick>,
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<crate::ingame::input::State>,
) {
    if ignore.0 {
        return;
    }
    let_some!(w = &w.0, return);
    let_some!(cursor = window.cursor_position(), return);
    let (camera, transform) = *camera;
    let point = camera.viewport_to_world_2d(transform, cursor);
    let_ok!(point = point, return);

    let LeftClickMode::LineOfView(profile) = &mode.0 else {
        return;
    };

    return_if!(maybe_cancel(&mut commands, &buttons, &keys));
    show(w, point, &mut commands, &buttons, &mut state, &profile);
}

pub fn show(
    _w: &WorldConfig,
    point: Vec2,
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
                    start_pluz_z: profile.start_pluz_z,
                    stop_pluz_z: profile.stop_pluz_z,
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
