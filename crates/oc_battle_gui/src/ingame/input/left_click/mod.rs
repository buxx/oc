use bevy::color::palettes::css::YELLOW;
use bevy::prelude::*;
use enum_type_derive::EnumType;
use oc_individual::order::OrderType;
use oc_root::Wcfg;
use oc_utils::{let_ok, let_some};
use strum_macros::EnumIter;

use crate::ingame;
#[cfg(feature = "debug")]
use crate::ingame::debug::projectile::SpawnProjectileProfile;
use crate::ingame::draw;
#[cfg(feature = "debug")]
use crate::ingame::lov::SpawnProjectileClickMode;
use crate::ingame::lov::{DespawnLov, LovClickMode, SpawnLov, SpawnLovConfig, SpawnLovProfile};
use crate::window::PointerInWindow;
use crate::world::World;

#[cfg(feature = "debug")]
pub mod spawn_projectile;

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

pub fn show(
    mut commands: Commands,
    w: Res<Wcfg>,
    ignore: Res<PointerInWindow>,
    window: Single<&Window>,
    camera: Single<(&Camera, &GlobalTransform)>,
    buttons: Res<ButtonInput<MouseButton>>,
    mode: Res<LeftClick>,
    #[cfg(feature = "debug")] spawn_projectile_mode: Res<SpawnProjectileLeftClick>,
    _keys: Res<ButtonInput<KeyCode>>,
    mut ingame: ResMut<ingame::state::State>,
    mut state: ResMut<ingame::input::State>,
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
        LeftClickMode::SpawnProjectile(profile) => spawn_projectile::on_click(
            w,
            point,
            &mut commands,
            &buttons,
            &spawn_projectile_mode,
            &mut ingame,
            &mut state,
            &world,
            profile,
        ),
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
