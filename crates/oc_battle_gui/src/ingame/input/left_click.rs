#[cfg(feature = "debug")]
use crate::{
    ingame::debug::projectile::SpawnProjectileProfile,
    window::debug::battle::SpawnProjectileClickMode,
};
use bevy::{color::palettes::css::YELLOW, prelude::*};
use oc_network::ToServer;
use oc_projectile::spawn::SpawnProjectile;
use strum_macros::EnumIter;

use crate::{ingame::draw, network::output::ToServerEvent, window::PointerInWindow};

#[derive(Debug, Deref, DerefMut, Event)]
pub struct SetLeftClick(pub LeftClickMode);

#[cfg(feature = "debug")]
#[derive(Debug, Deref, DerefMut, Event)]
pub struct SetSpawnProjectileLeftClickMode(pub SpawnProjectileClickMode);

#[derive(Debug, Event)]
pub struct SpawnClicksLine;

#[derive(Debug, Event)]
pub struct DespawnClicksLine;

#[derive(Debug, Component)]
pub struct ClicksLine;

#[derive(Debug, Deref, DerefMut, Resource, Default)]
pub struct LeftClick(pub LeftClickMode);

#[cfg(feature = "debug")]
#[derive(Debug, Deref, DerefMut, Resource, Default)]
pub struct SpawnProjectileLeftClick(pub SpawnProjectileClickMode);

#[derive(Debug, Clone, Default)]
pub enum LeftClickMode {
    #[default]
    Select,
    #[cfg(feature = "debug")]
    SpawnProjectile(SpawnProjectileProfile),
}

#[derive(Debug, Clone, Default, PartialEq, Eq, EnumIter)]
pub enum LeftClickModeType {
    #[default]
    Select,
    #[cfg(feature = "debug")]
    SpawnProjectile,
}

impl LeftClickModeType {
    pub fn name(&self) -> &str {
        match self {
            LeftClickModeType::Select => "Select",
            LeftClickModeType::SpawnProjectile => "Spawn projectile",
        }
    }
}

pub fn click(
    mut commands: Commands,
    ignore: Res<PointerInWindow>,
    window: Single<&Window>,
    camera: Single<(&Camera, &GlobalTransform)>,
    buttons: Res<ButtonInput<MouseButton>>,
    mode: Res<LeftClick>,
    spawn_mode: Res<SpawnProjectileLeftClick>,
    _keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<super::State>,
) {
    if ignore.0 {
        return;
    }
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let (camera, transform) = *camera;
    let Ok(point) = camera.viewport_to_world_2d(transform, cursor) else {
        return;
    };

    match &mode.0 {
        LeftClickMode::Select => {
            // TODO
        }
        LeftClickMode::SpawnProjectile(profile) => match spawn_mode.0 {
            SpawnProjectileClickMode::TwoClicks => {
                if buttons.just_released(MouseButton::Left) {
                    state.clicks.push(point);

                    if state.clicks.len() == 1 {
                        commands.trigger(SpawnClicksLine);
                    }

                    if state.clicks.len() == 2 {
                        let start = state.clicks.first().expect("len checked line before");
                        let start = [start.x, start.y];
                        let end = state.clicks.last().expect("len checked line before");
                        let end = [end.x, end.y];
                        let projectile = profile.projectile.id();
                        let profile = profile.profile.clone();
                        let spawn = SpawnProjectile::new(projectile, profile, start, end);

                        commands.trigger(ToServerEvent(ToServer::SpawnProjectile(spawn)));
                        commands.trigger(DespawnClicksLine);

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
                        let start = [start.x, start.y];
                        let end = [point.x, point.y];

                        let projectile = profile.projectile.id();
                        let profile = profile.profile.clone();
                        let spawn = SpawnProjectile::new(projectile, profile, start, end);

                        commands.trigger(ToServerEvent(ToServer::SpawnProjectile(spawn)));
                    }

                    commands.trigger(DespawnClicksLine);
                    state.clicks.clear();
                }
            }
        },
    }

    //
}

pub fn on_set_left_click(set: On<SetLeftClick>, mut left_click: ResMut<LeftClick>) {
    left_click.0 = set.0.clone();
}

pub fn on_set_spawn_projectile_left_click(
    set: On<SetSpawnProjectileLeftClickMode>,
    mut left_click: ResMut<SpawnProjectileLeftClick>,
) {
    left_click.0 = set.0.clone();
}

pub fn on_spawn_clicks_line(
    _: On<SpawnClicksLine>,
    window: Single<&Window>,
    camera: Single<(&Camera, &GlobalTransform)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    state: Res<super::State>,
) {
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let (camera, transform) = *camera;
    let Ok(point) = camera.viewport_to_world_2d(transform, cursor) else {
        return;
    };

    let mut points = state.clicks.clone();
    points.push(point);
    let line = Polyline2d::new(points);

    commands.spawn((
        ClicksLine,
        Mesh2d(meshes.add(line)),
        MeshMaterial2d(materials.add(Color::from(YELLOW))),
        Transform::from_xyz(0., 0., draw::Z_SELECT_WIRES),
    ));
}

pub fn update_clicks_line(mut commands: Commands, mode: Res<LeftClick>, state: Res<super::State>) {
    match &mode.0 {
        LeftClickMode::Select => {}
        LeftClickMode::SpawnProjectile(_) => {
            if !state.clicks.is_empty() {
                commands.trigger(DespawnClicksLine);
                commands.trigger(SpawnClicksLine);
            }
        }
    }
}

pub fn on_despawn_clicks_line(
    _: On<DespawnClicksLine>,
    mut commands: Commands,
    line: Single<Entity, With<ClicksLine>>,
) {
    commands.entity(line.into_inner()).despawn();
}
