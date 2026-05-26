use bevy::prelude::*;

use derive_more::Deref;
use oc_geo::tile::TileXy;
#[cfg(feature = "debug")]
use oc_mod::DEFAULT_HUMAN_DEFAULT_STAND_UP_FIRE_METERS;
use oc_root::{Wcfg, WorldConfig, physics::Meters, y::Y};
use oc_utils::d2::Xy;

#[cfg(feature = "debug")]
use crate::ingame::input::left_click::{LeftClick, LeftClickMode};

#[cfg(feature = "debug")]
use crate::window::debug::battle::LovClickMode;

use crate::{
    ingame::draw,
    states::{InGameState, Mod},
    world::World,
};

#[derive(Debug, Event, Deref)]
pub struct SpawnLov(pub SpawnLovProfile);

#[derive(Debug, Event)]
pub struct UpdateLovFor(pub Entity, pub Vec2);

#[derive(Debug, Event)]
pub struct DespawnLov;

#[cfg(feature = "debug")]
#[derive(Debug, Clone)]
pub struct SpawnLovConfig {
    #[cfg(feature = "debug")]
    pub click: LovClickMode,
    pub start_pluz_z: Meters,
    pub stop_pluz_z: Meters,
}

#[cfg(feature = "debug")]
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
    let Some(w) = &w.0 else { return };
    let Some(tile) = world.tile_at(w, &spawn.start.to_gui_y(w)) else {
        return;
    };
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
    #[cfg(feature = "debug")] mode: Res<LeftClick>,
) {
    #[cfg(feature = "debug")]
    if !matches!(mode.0, LeftClickMode::LineOfView(_)) {
        return;
    }
    let (camera, transform) = *camera;
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let Ok(position) = camera.viewport_to_world_2d(transform, cursor) else {
        return;
    };

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
    w: Res<Wcfg>,
    mod_: Res<Mod>,
    mut lovs: Query<&mut Lov>,
    world: Res<World>,
) {
    tracing::trace!(name = "on-update-lov-for-try");
    let Some(w) = &w.0 else {
        tracing::trace!(name = "on-update-lov-for-no-w");
        return;
    };
    let Some(mod_) = &mod_.0 else {
        tracing::trace!(name = "on-update-lov-for-no-mod");
        return;
    };
    let (lov, position) = (update.0, update.1);
    let Ok(mut lov) = lovs.get_mut(lov) else {
        tracing::trace!(name = "on-update-lov-for-no-entity");
        return;
    };

    let start = lov.start;
    let start_ = [start.x, start.y.to_gui_y(w), start.z];
    let Some(stop_tile) = world.tile_at(w, &position.to_gui_y(w)) else {
        tracing::trace!(name = "on-update-lov-for-no-tile", position=?position.to_gui_y(w));
        return;
    };
    tracing::trace!(name = "on-update-lov-for");
    let stop = position.extend(stop_tile.z_pixels(w) + lov.stop_plus_z.pixels(w));
    let end_ = [stop.x, stop.y.to_gui_y(w), stop.z];
    let at = |xy, z| path_objects_at(w, mod_, &world, xy, z);
    let path = oc_lov::PathBuilder::new(w, at, w.geo_lov_step).build_(start_, end_);

    let sections = path.sections.iter().map(|section| {
        let color = Color::srgb(0.0 + section.opacity.0, 1.0 - section.opacity.0, 0.0);
        let start = Vec2::new(section.start[0], section.start[1].to_gui_y(w));
        let stop = Vec2::new(section.stop[0], section.stop[1].to_gui_y(w));
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
    for (entity, lov) in lovs {
        println!("{:?} -> {:?}", lov.start, lov.stop);
        commands.entity(entity).despawn();
    }
}
