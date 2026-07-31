use bevy::color::palettes::css::YELLOW;
use bevy::prelude::*;
use enum_type_derive::EnumType;
use oc_individual::order::OrderType;
use oc_network::ToServer;
use oc_root::Wcfg;
use oc_root::physics::Meters;
use oc_utils::{let_ok, let_some};
#[cfg(feature = "debug")]
use strum_macros::EnumIter;

#[cfg(feature = "debug")]
use crate::ingame::debug::projectile::SpawnProjectileProfile;
#[cfg(feature = "debug")]
use crate::ingame::lov::SpawnProjectileClickMode;
use crate::ingame::lov::{DespawnLov, LovClickMode, SpawnLov, SpawnLovConfig, SpawnLovProfile};
use crate::window::PointerInWindow;
use crate::world::World;
use crate::{ingame::draw, network::output::ToServerEvent};

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

#[derive(Debug, Deref, DerefMut, Resource)]
pub struct LeftClick(pub LeftClickMode);

impl Default for LeftClick {
    fn default() -> Self {
        Self(LeftClickMode::Select)
    }
}

#[cfg(feature = "debug")]
#[derive(Debug, Deref, DerefMut, Resource, Default)]
pub struct SpawnProjectileLeftClick(pub SpawnProjectileClickMode);

#[derive(Debug, Clone, EnumType)]
#[enum_type(derive(EnumIter))]
pub enum LeftClickMode {
    Select,
    #[cfg(feature = "debug")]
    SpawnProjectile(SpawnProjectileProfile),
    LineOfView(SpawnLovConfig),
    Order(OrderType),
}

impl LeftClickModeType {
    pub fn name(&self) -> &str {
        match self {
            LeftClickModeType::Select => "Select",
            #[cfg(feature = "debug")]
            LeftClickModeType::SpawnProjectile => "Spawn projectile",
            LeftClickModeType::LineOfView => "Line of view",
            LeftClickModeType::Order => "Order",
        }
    }
}

pub fn click_debug(
    mut commands: Commands,
    w: Res<Wcfg>,
    ignore: Res<PointerInWindow>,
    window: Single<&Window>,
    camera: Single<(&Camera, &GlobalTransform)>,
    buttons: Res<ButtonInput<MouseButton>>,
    mode: Res<LeftClick>,
    #[cfg(feature = "debug")] spawn_projectile_mode: Res<SpawnProjectileLeftClick>,
    _keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<super::State>,
    #[cfg(feature = "debug")] world: Res<World>,
) {
    if ignore.0 {
        return;
    }
    let_some!(w = &w.0, return);
    let_some!(cursor = window.cursor_position(), return);
    let (camera, transform) = *camera;
    let point = camera.viewport_to_world_2d(transform, cursor);
    let_ok!(point = point, return);

    match &mode.0 {
        LeftClickMode::Select => {
            // TODO
        }
        LeftClickMode::Order(_) => {
            // TODO
        }

        LeftClickMode::LineOfView(profile) => match profile.click {
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
        },

        #[cfg(feature = "debug")]
        LeftClickMode::SpawnProjectile(profile) => match spawn_projectile_mode.0 {
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
        },
    }

    //
}

pub fn on_set_left_click(set: On<SetLeftClick>, mut left_click: ResMut<LeftClick>) {
    left_click.0 = set.0.clone();
}

#[cfg(feature = "debug")]
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
    let_some!(cursor = window.cursor_position(), return);
    let (camera, transform) = *camera;
    let point = camera.viewport_to_world_2d(transform, cursor);
    let_ok!(point = point, return);

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

pub fn update_spawn_projectile_clicks_line(
    mut commands: Commands,
    mode: Res<LeftClick>,
    state: Res<super::State>,
) {
    match &mode.0 {
        LeftClickMode::Select | LeftClickMode::LineOfView(_) | LeftClickMode::Order(_) => {}
        #[cfg(feature = "debug")]
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
