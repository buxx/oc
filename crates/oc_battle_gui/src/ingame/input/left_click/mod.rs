#[cfg(feature = "debug")]
use bevy::color::palettes::css::YELLOW;
use bevy::prelude::*;
use enum_type_derive::EnumType;
use oc_geo::tile::TileXy;
use oc_geo::tile::WorldTileIndex;
use oc_individual::order::OrderType;
use oc_root::Wcfg;
use oc_root::WcfgFrom;
use oc_root::y::Y;
use oc_utils::{let_ok, let_some};
use strum_macros::EnumIter;

use crate::ingame;
#[cfg(feature = "debug")]
use crate::ingame::debug::projectile::SpawnProjectileProfile;
#[cfg(feature = "debug")]
use crate::ingame::draw;
#[cfg(feature = "debug")]
use crate::ingame::lov::SpawnLovConfig;
#[cfg(feature = "debug")]
use crate::ingame::lov::SpawnProjectileClickMode;
use crate::ingame::path::ComputeDisplayPaths;
use crate::ingame::path::SpawnPathProfile;
use crate::ingame::path::SpawnPathProfileKey;
use crate::window::PointerInWindow;
use crate::world::World;

#[cfg(feature = "debug")]
pub mod lov;
pub mod order;
#[cfg(feature = "debug")]
pub mod spawn_projectile;

#[derive(Debug, Deref, DerefMut, Event)]
pub struct SetLeftClick(pub LeftClickMode);

#[cfg(feature = "debug")]
#[derive(Debug, Deref, DerefMut, Event)]
pub struct SetSpawnProjectileLeftClickMode(pub SpawnProjectileClickMode);

#[cfg(feature = "debug")]
#[derive(Debug, Event)]
pub struct SpawnClicksLine;

#[cfg(feature = "debug")]
#[derive(Debug, Event)]
pub struct DespawnClicksLine;

#[cfg(feature = "debug")]
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
    ///The default mode which is selector
    Select,
    /// For debug, ability to spawn projectile from mouse
    #[cfg(feature = "debug")]
    SpawnProjectile(SpawnProjectileProfile),
    // FIXME BS NOW For now only in debug window, but need possibility to player to use it
    /// User can see a line of view from arbitrary place
    #[cfg(feature = "debug")]
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
            #[cfg(feature = "debug")]
            LeftClickMode::LineOfView(_) => true,
            LeftClickMode::Order(order) => match order {
                OrderType::Idle => false,
                // FIXME BS NOW
                OrderType::MoveTo => true,
            },
        }
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

pub fn show(
    mut commands: Commands,
    w: Res<Wcfg>,
    ignore: Res<PointerInWindow>,
    window: Single<&Window>,
    camera: Single<(&Camera, &GlobalTransform)>,
    #[cfg(feature = "debug")] buttons: Res<ButtonInput<MouseButton>>,
    mode: Res<LeftClick>,
    #[cfg(feature = "debug")] spawn_projectile_mode: Res<SpawnProjectileLeftClick>,
    _keys: Res<ButtonInput<KeyCode>>,
    #[cfg(feature = "debug")] mut ingame: ResMut<ingame::state::State>,
    #[cfg(not(feature = "debug"))] ingame: ResMut<ingame::state::State>,
    #[cfg(feature = "debug")] mut state: ResMut<ingame::input::State>,
    world: Res<World>,
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
        // FIXME BS NOW: move code into order.rs
        LeftClickMode::Order(order) => {
            tracing::trace!(name="ingame-input-left_click-show-order", mode=?mode.0, point=?point);

            match order {
                OrderType::Idle => {}
                OrderType::MoveTo => {
                    let spawns = ingame.selected_squads().iter().filter_map(|i| {
                        tracing::trace!(name="ingame-input-left_click-show-order-squad", mode=?mode.0, point=?point, squad=?i);
                        let squad = world.squad(i)?;
                        let leader = world.get_individual(squad.leader())?;
                        let start = Vec2::new(leader.position[0], leader.position[1]);
                        let start = start.to_gui_y(w);
                        let end = point;
                        let start_tile = TileXy::from_([start.x, start.y], w);
                        let start_tile = WorldTileIndex::from_(start_tile, w);
                        let key = SpawnPathProfileKey::Squad{ i: *i, start: start_tile, end };
                        Some(SpawnPathProfile { key, start, end })
                    }).collect::<Vec<SpawnPathProfile>>();
                    // FIXME BS NOW: do not recalculate when same ? Currently 100% CPU
                    commands.trigger(ComputeDisplayPaths(spawns));
                } // OrderType::Fire => {
                  //     commands.trigger(SpawnLov(SpawnLovProfile {
                  //         start,
                  //         // FIXME BS NOW: individual (squad leader) weapons z (according to gesture)
                  //         start_pluz_z: Meters(1.),
                  //         // Ground z if no body, body z (according to gesture) if target under cursor
                  //         stop_pluz_z: Meters(1.),
                  //     }));
                  // },
            }
        }

        #[cfg(feature = "debug")]
        LeftClickMode::LineOfView(profile) => {
            tracing::trace!(name="ingame-input-left_click-show-lov", mode=?mode.0, point=?point);
            lov::show(point, &mut commands, &buttons, &mut state, profile);
        }

        #[cfg(feature = "debug")]
        LeftClickMode::SpawnProjectile(profile) => {
            tracing::trace!(name="ingame-input-left_click-show-spawn-projectile", mode=?mode.0, point=?point);
            spawn_projectile::show(
                w,
                point,
                &mut commands,
                &buttons,
                &spawn_projectile_mode,
                &mut ingame,
                &mut state,
                &world,
                profile,
            )
        }
    }

    //
}

#[cfg(feature = "debug")]
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

#[cfg(feature = "debug")]
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

#[cfg(feature = "debug")]
pub fn on_despawn_clicks_line(
    _: On<DespawnClicksLine>,
    mut commands: Commands,
    line: Single<Entity, With<ClicksLine>>,
) {
    commands.entity(line.into_inner()).despawn();
}
