use bevy::prelude::*;
use oc_network::ToServer;
use oc_root::WorldConfig;
use oc_root::physics::Meters;

use crate::ingame;
use crate::ingame::debug::projectile::SpawnProjectileProfile;
use crate::ingame::input::left_click::{
    DespawnClicksLine, SpawnClicksLine, SpawnProjectileLeftClick,
};
use crate::ingame::lov::SpawnProjectileClickMode;
use crate::network::output::ToServerEvent;
use crate::world::World;

pub fn on_click(
    w: &WorldConfig,
    point: Vec2,
    commands: &mut Commands,
    buttons: &ButtonInput<MouseButton>,
    spawn_projectile_mode: &SpawnProjectileLeftClick,
    _ingame: &mut ingame::state::State,
    state: &mut ingame::input::State,
    world: &World,
    profile: &SpawnProjectileProfile,
) {
    match spawn_projectile_mode.0 {
        SpawnProjectileClickMode::TwoClicks => {
            if buttons.just_released(MouseButton::Left) {
                state.clicks.push(point);

                if state.clicks.len() == 1 {
                    commands.trigger(SpawnClicksLine);
                }

                if state.clicks.len() == 2 {
                    let start = state.clicks.first().expect("len checked line before");
                    let end = state.clicks.last().expect("len checked line before");

                    if let (Some(start), Some(end)) = (
                        world.point2d_to_point3d(w, start, profile.plus_z),
                        world.point2d_to_point3d(w, &end, Meters(0.)),
                    ) {
                        use crate::projectile::IntoSpawnProjectile;
                        let spawn = profile.spawn(start, end);
                        tracing::debug!("Spawn projectile {spawn:?}");
                        commands.trigger(ToServerEvent(ToServer::SpawnProjectile(spawn)));
                    }

                    state.clicks.clear();
                }
            }
        }
        SpawnProjectileClickMode::DraggedClick => {
            if buttons.just_pressed(MouseButton::Left) {
                state.clicks.push(point);
                commands.trigger(SpawnClicksLine);
            }

            if buttons.just_released(MouseButton::Left) {
                if let Some(start) = state.clicks.first() {
                    if let (Some(start), Some(end)) = (
                        world.point2d_to_point3d(w, start, profile.plus_z),
                        world.point2d_to_point3d(w, &point, Meters(0.)),
                    ) {
                        use crate::projectile::IntoSpawnProjectile;
                        let spawn = profile.spawn(start, end);
                        tracing::debug!("Spawn projectile {spawn:?}");
                        commands.trigger(ToServerEvent(ToServer::SpawnProjectile(spawn)));
                    }
                }

                commands.trigger(DespawnClicksLine);
                state.clicks.clear();
            }
        }
    }
}
