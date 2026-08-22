use std::time::Duration;

use bevy::prelude::*;

use derive_more::Constructor;
use oc_mod::{ammunition::AmmunitionIndex, armament::ShotModeIndex, weapons::WeaponIndex};
use oc_root::{physics::Meters, side::Side, y::Y};
use oc_utils::let_some;

use crate::states::GameConfig;

#[derive(Debug, Clone, Constructor)]
pub struct SpawnProjectileProfile {
    pub weapon: WeaponIndex,
    pub ammunition: AmmunitionIndex,
    pub shot: ShotModeIndex,
    pub repeat: u8,
    pub plus_z: Meters,
    pub side: Side,
}

#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct CollisionGizmos;

pub fn setup(mut config: ResMut<GizmoConfigStore>) {
    tracing::trace!(name = "ingame-debug-projectiles-setup");
    let (gizmos, _) = config.config_mut::<CollisionGizmos>();
    gizmos.line.width = 2.0;
}

pub fn show_collisions(
    g: Res<GameConfig>,
    mut state: ResMut<crate::ingame::state::State>,
    mut gizmos: Gizmos<CollisionGizmos>,
) {
    let_some!(g = &g.0, return);
    const DISPLAY_DURATION: Duration = Duration::from_millis(500);
    const COLOR: Color = Color::linear_rgb(1.0, 1.0, 0.0);

    let mut collisions = Vec::with_capacity(state.collisions.len());
    while let Some((instant, position)) = state.collisions.pop() {
        if instant.elapsed() <= DISPLAY_DURATION {
            let position_ = Vec2::new(position.x, position.y.to_gui_y(&g.w));
            gizmos.circle_2d(position_, 1.0, COLOR);
            collisions.push((instant, position))
        }
    }
    state.collisions = collisions;
}
