use bevy::prelude::*;

use crate::ingame;
use crate::ingame::lov::{DespawnLov, LovClickMode, SpawnLov, SpawnLovConfig, SpawnLovProfile};

pub fn show(
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
