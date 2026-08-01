use bevy::prelude::*;

use derive_more::Deref;
use oc_geo::tile::TileXy;
use oc_mod::DEFAULT_HUMAN_DEFAULT_STAND_UP_FIRE_METERS;
use oc_root::{Wcfg, WorldConfig, physics::Meters, y::Y};
use oc_utils::{d2::Xy, let_ok, let_some};
use strum_macros::{Display, EnumIter};

use crate::ingame::input::left_click::{LeftClick, LeftClickMode};

use crate::states::GameConfig;

use crate::{ingame::draw, states::InGameState, world::World};

#[derive(Debug, Event, Deref)]
pub struct SpawnLov(pub SpawnLovProfile);

#[derive(Debug, Event)]
pub struct UpdateLovFor(pub Entity, pub Vec2);

#[derive(Debug, Event)]
pub struct DespawnLov;

#[derive(Debug, Clone)]
pub struct SpawnLovConfig {
    pub click: LovClickMode,
    pub start_pluz_z: Meters,
    pub stop_pluz_z: Meters,
}

impl Default for SpawnLovConfig {
    fn default() -> Self {
        Self {
            click: LovClickMode::DraggedClick,
            start_pluz_z: DEFAULT_HUMAN_DEFAULT_STAND_UP_FIRE_METERS,
            stop_pluz_z: Meters(0.),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SpawnLovProfile {
    pub start: Vec2,
    pub start_pluz_z: Meters,
    pub stop_pluz_z: Meters,
}

#[derive(Debug, Component)]
pub struct Lov {
    // pub steps: Vec<(Vec3, CumulatedOpacity)>,
    pub start: Vec3,
    pub stop: Vec3,
    pub stop_plus_z: Meters,
    pub sections: Vec<(Vec2, Vec2, Color)>,
}

#[derive(Debug, Clone, Copy, Default, Display, EnumIter, PartialEq, Eq)]
pub enum SpawnProjectileClickMode {
    TwoClicks,
    #[default]
    DraggedClick,
}

#[derive(Debug, Clone, Copy, Default, Display, EnumIter, PartialEq, Eq)]
pub enum LovClickMode {
    TwoClicks,
    #[default]
    DraggedClick,
}

#[derive(Default, Reflect, GizmoConfigGroup)]
struct LovGizmos;

pub struct LovPlugin;

impl Plugin for LovPlugin {
    fn build(&self, app: &mut App) {
        app.init_gizmo_group::<LovGizmos>()
            .add_systems(Startup, setup)
            .add_observer(on_spawn_lov)
            .add_observer(on_update_lov_for)
            .add_observer(on_despawn_lov)
            .add_systems(
                Update,
                (update_lov, draw_lovs).run_if(in_state(InGameState::Battle)),
            );
    }
}

fn setup(mut config: ResMut<GizmoConfigStore>) {
    let (gizmos, _) = config.config_mut::<LovGizmos>();
    gizmos.line.width = 5.0;
}

fn on_spawn_lov(spawn: On<SpawnLov>, w: Res<Wcfg>, mut commands: Commands, world: Res<World>) {
    tracing::trace!(name = "lov-spawn", spawn=?spawn);
    let_some!(w = &w.0, return);
    let_some!(tile = world.tile_at(w, &spawn.start.to_gui_y(w)), return);
    let z = tile.z_pixels(w) + spawn.start_pluz_z.0 * w.geo_pixels_per_meters;
    let start = spawn.start.extend(z);

    tracing::trace!(name = "lov-spawn", start=?start);
    commands.spawn(Lov {
        start,
        stop: start,
        stop_plus_z: spawn.stop_pluz_z,
        sections: vec![],
    });
}

fn update_lov(
    mut commands: Commands,
    lovs: Query<Entity, With<Lov>>,
    camera: Single<(&Camera, &GlobalTransform)>,
    window: Single<&Window>,
    mode: Res<LeftClick>,
) {
    if !mode.0.display_lov() {
        return;
    }
    let (camera, transform) = *camera;
    let_some!(cursor = window.cursor_position(), return);
    let point = camera.viewport_to_world_2d(transform, cursor);
    let_ok!(position = point, return);

    for lov in lovs {
        tracing::trace!(name="update-lov-trigger-for", lov=?lov, position=?position);
        commands.trigger(UpdateLovFor(lov, position));
    }
}

fn draw_lovs(lovs: Query<&Lov>, mut gizmos: Gizmos) {
    for lov in lovs {
        for (start, stop, color) in &lov.sections {
            gizmos.line(start.extend(draw::Z_LOV), stop.extend(draw::Z_LOV), *color);
        }
    }
}

fn on_update_lov_for(
    update: On<UpdateLovFor>,
    g: Res<GameConfig>,
    mut lovs: Query<&mut Lov>,
    world: Res<World>,
) {
    tracing::trace!(name = "on-update-lov-for-try");
    let_some!(g = &g.0, return);
    let (lov, position) = (update.0, update.1);
    let_ok!(mut lov = lovs.get_mut(lov), return);

    let start = lov.start;
    let start_ = [start.x, start.y.to_gui_y(&g.w), start.z];
    let stop_tile = world.tile_at(&g.w, &position.to_gui_y(&g.w));
    let_some!(stop_tile = stop_tile, return);
    tracing::trace!(name = "on-update-lov-for");

    let stop = position.extend(stop_tile.z_pixels(&g.w) + lov.stop_plus_z.pixels(&g.w));
    let end_ = [stop.x, stop.y.to_gui_y(&g.w), stop.z];
    let at = |xy, z| path_objects_at(&g.w, &g.mod_, &world, xy, z);
    let path = oc_lov::PathBuilder::new(&g.w, at).build_(start_, end_);

    let sections = path.sections.iter().map(|section| {
        let color = Color::srgb(0.0 + section.opacity.0, 1.0 - section.opacity.0, 0.0);
        let start = Vec2::new(section.start[0], section.start[1].to_gui_y(&g.w));
        let stop = Vec2::new(section.stop[0], section.stop[1].to_gui_y(&g.w));
        tracing::trace!(name = "on-update-lov-for-section", start=?start, stop=?stop, color=?color);
        (start, stop, color)
    }).collect();

    lov.start = start;
    lov.stop = stop;
    lov.sections = sections;
}

fn path_objects_at(
    w: &WorldConfig,
    mod_: &oc_mod::Mod,
    world: &World,
    at: Xy,
    z: f32,
) -> Vec<oc_lov::Step> {
    world
        .tile(w, TileXy(at))
        .map(|t| {
            let tile_z = t.z as f32 * w.geo_meters_per_z.0 * w.geo_pixels_per_meters;
            let relative_z = z - tile_z;
            let opacity = mod_.nature(t.nature).opacity(w, relative_z);
            vec![oc_lov::Step { opacity }]
        })
        .unwrap_or(vec![])
}

fn on_despawn_lov(_: On<DespawnLov>, mut commands: Commands, lovs: Query<(Entity, &Lov)>) {
    tracing::trace!(name = "lov-despawn");
    for (entity, _lov) in lovs {
        commands.entity(entity).despawn();
    }
}
