use bevy::color::palettes::css::YELLOW;
use bevy::prelude::*;
use derive_is_enum_variant::is_enum_variant;
use enum_type_derive::EnumType;
use oc_individual::order::OrderType;
use oc_root::WcfgFrom;
use oc_root::geo::ScreenVec2;
use oc_root::geo::WorldVec2;
use oc_utils::{let_ok, let_some};
use strum_macros::EnumIter;

#[cfg(feature = "debug")]
use crate::ingame::debug::projectile::SpawnProjectileProfile;
use crate::ingame::draw;
use crate::ingame::lov::SpawnLovConfig;
#[cfg(feature = "debug")]
use crate::ingame::lov::SpawnProjectileClickMode;
use crate::states::GameConfig;

pub mod lov;
pub mod order;
pub mod select;
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

#[derive(Debug, Clone, EnumType, is_enum_variant)]
#[enum_type(derive(EnumIter, States))]
pub enum LeftClickMode {
    ///The default mode which is selector
    Select,
    /// For debug, ability to spawn projectile from mouse
    #[cfg(feature = "debug")]
    SpawnProjectile(SpawnProjectileProfile),
    /// User can see a line of view from arbitrary place
    LineOfView(SpawnLovConfig),
    /// User is going to give a squad order
    Order(OrderType),
}

impl LeftClickMode {
    pub fn display_lov(&self) -> bool {
        match self {
            LeftClickMode::Select => false,
            #[cfg(feature = "debug")]
            LeftClickMode::SpawnProjectile(_) => false,
            LeftClickMode::LineOfView(_) => true,
            LeftClickMode::Order(order) => match order {
                OrderType::Idle => false,
                // FIXME BS NOW
                OrderType::MoveTo => true,
            },
        }
    }
}

impl Default for LeftClickModeType {
    fn default() -> Self {
        Self::Select
    }
}

impl LeftClickModeType {
    #[cfg(feature = "debug")]
    pub fn name(&self) -> &str {
        match self {
            LeftClickModeType::Select => "Select",
            LeftClickModeType::SpawnProjectile => "Spawn projectile",
            LeftClickModeType::LineOfView => "Line of view",
            LeftClickModeType::Order => "Order",
        }
    }
}

pub fn on_set_left_click(
    set: On<SetLeftClick>,
    mut left_click: ResMut<LeftClick>,
    mut state: ResMut<NextState<LeftClickModeType>>,
) {
    left_click.0 = set.0.clone();
    *state = NextState::Pending((&set.0).into());
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
    g: Res<GameConfig>,
    window: Single<&Window>,
    camera: Single<(&Camera, &GlobalTransform)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    state: Res<super::State>,
) {
    let_some!(g = &g.0, return);
    let_some!(cursor = window.cursor_position(), return);
    let (camera, transform) = *camera;
    let point = camera.viewport_to_world_2d(transform, cursor);
    let_ok!(point = point, return);
    let point = ScreenVec2::new(point.x, point.y);
    let point = WorldVec2::from_(point, &g.w);

    let mut points = state.clicks.clone();
    points.push(point);
    let vertices: Vec<Vec2> = points
        .into_iter()
        .map(|p| ScreenVec2::from_(p, &g.w))
        .map(|p| Vec2::new(p.x, p.y))
        .collect();
    let line = Polyline2d::new(vertices);

    commands.spawn((
        ClicksLine,
        Mesh2d(meshes.add(line)),
        MeshMaterial2d(materials.add(Color::from(YELLOW))),
        Transform::from_xyz(0., 0., draw::Z_SELECT_WIRES),
    ));
}

#[cfg(feature = "debug")]
pub fn update_spawn_projectile_clicks_line(
    mut commands: Commands,
    mode: Res<LeftClick>,
    state: Res<super::State>,
) {
    match &mode.0 {
        LeftClickMode::Select | LeftClickMode::LineOfView(_) | LeftClickMode::Order(_) => {}
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
