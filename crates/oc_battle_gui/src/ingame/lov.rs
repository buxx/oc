use bevy::prelude::*;

use derive_more::Deref;
use oc_geo::tile::TileXy;
use oc_root::{Wcfg, WorldConfig, opacity::Opacity, physics::Meters};
use oc_utils::d2::Xy;

use crate::{
    ingame::draw,
    states::{InGameState, Mod},
    window::debug::battle::LovClickMode,
    world::World,
};

#[derive(Debug, Event, Deref)]
pub struct SpawnLov(pub SpawnLovProfile);

#[derive(Debug, Event)]
pub struct DespawnLov;

#[derive(Debug, Clone, Default)]
pub struct SpawnLovConfig {
    pub click: LovClickMode,
    pub start_pluz_z: Meters,
    pub stop_pluz_z: Meters,
}

#[derive(Debug, Clone)]
pub struct SpawnLovProfile {
    pub start: Vec2,
    pub start_pluz_z: Meters,
    pub stop_pluz_z: Meters,
}

#[derive(Debug, Resource, Deref, DerefMut, Default)]
pub struct Lov(pub Option<Lov_>);

#[derive(Debug)]
pub struct Lov_ {
    // pub steps: Vec<(Vec3, CumulatedOpacity)>,
    pub start: Vec3,
    pub stop: Vec3,
    pub stop_plus_z: Meters,
}

#[derive(Default, Reflect, GizmoConfigGroup)]
struct LovGizmos;

pub struct LovPlugin;

impl Plugin for LovPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Lov>()
            .init_gizmo_group::<LovGizmos>()
            .add_systems(Startup, setup)
            .add_observer(on_spawn_lov)
            .add_observer(on_despawn_lov)
            .add_systems(Update, update_lov.run_if(in_state(InGameState::Battle)));
    }
}

fn setup(mut config: ResMut<GizmoConfigStore>) {
    let (gizmos, _) = config.config_mut::<LovGizmos>();
    gizmos.line.width = 5.0;
}

pub fn on_spawn_lov(spawn: On<SpawnLov>, w: Res<Wcfg>, mut lov: ResMut<Lov>, world: Res<World>) {
    tracing::trace!(name = "lov-spawn", spawn=?spawn);
    let Some(w) = &w.0 else { return };
    let Some(tile) = world.tile_at(w, &spawn.start) else {
        return;
    };
    let z = tile.z_pixels(w);
    let start = spawn.start.extend(z);
    tracing::trace!(name = "lov-spawn", start=?start);
    lov.0 = Some(Lov_ {
        // steps: vec![],
        start,
        stop: start,
        stop_plus_z: spawn.stop_pluz_z,
    });
}

pub fn update_lov(
    w: Res<Wcfg>,
    mod_: Res<Mod>,
    mut lov: ResMut<Lov>,
    camera: Single<(&Camera, &GlobalTransform)>,
    window: Single<&Window>,
    mut gizmos: Gizmos,
    world: Res<World>,
) {
    let Some(w) = &w.0 else { return };
    let Some(mod_) = &mod_.0 else { return };
    let Some(lov) = &mut lov.0 else { return };
    let (camera, transform) = *camera;
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let Ok(position) = camera.viewport_to_world_2d(transform, cursor) else {
        return;
    };

    let start = lov.start;
    let start_ = [start.x, start.y, start.z];
    let Some(stop_tile) = world.tile_at(w, &position) else {
        return;
    };
    let stop = position.extend(stop_tile.z_pixels(w) + lov.stop_plus_z.pixels(w));
    let end_ = [stop.x, stop.y, stop.z];
    // FIXME BS NOW: il faut prendre en compte z (+unit tests)
    let at = |xy, z| path_objects_at(w, mod_, &world, xy, z);
    let path = oc_lov::PathBuilder::new(w, at, w.geo_lov_step).build_(start_, end_);

    for section in path.sections {
        let color = Color::srgb(0.0 + section.opacity.0, 1.0 - section.opacity.0, 0.0);
        let start = Vec2::new(section.start[0], section.start[1]);
        let stop = Vec2::new(section.stop[0], section.stop[1]);
        gizmos.line(start.extend(draw::Z_LOV), stop.extend(draw::Z_LOV), color);
    }

    lov.start = start;
    lov.stop = stop;
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
            // FIXME BS NOW: 1.0 if "inside" (z + nature of tile, like wall, etc)
            // FIXME BS NOW z relative
            // FIXME BS NOW: le z du terrain Nature doit AUSSI influencer le volume (moteur physique) !!
            let opacity = mod_.terrain(t.nature.to_string()).opacity(0.);
            vec![oc_lov::Step { opacity }]
        })
        .unwrap_or(vec![])
}

pub fn on_despawn_lov(_: On<DespawnLov>, mut lov: ResMut<Lov>) {
    tracing::trace!(name = "lov-despawn");
    lov.0 = None;
}
