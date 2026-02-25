use bevy::prelude::*;
use oc_geo::region::RegionXy;
use oc_root::{GEO_PIXELS_PER_TILE, REGION_HEIGHT, REGION_WIDTH, WORLD_HEIGHT, WORLD_WIDTH};

use super::{ForgottenRegion, ListeningRegion};
use crate::entity::world::region::RegionWireFrame;
use crate::ingame::camera::map::{WORLD_MAP_X, WORLD_MAP_Y};
use crate::ingame::draw;

#[derive(Debug, Component)]
pub struct RegionWireFrameDebug;

pub fn on_listening_region(
    region: On<ListeningRegion>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    window: Single<&Window>,
) {
    // Battle display
    let width = REGION_WIDTH as f32 * GEO_PIXELS_PER_TILE as f32;
    let height = REGION_HEIGHT as f32 * GEO_PIXELS_PER_TILE as f32;
    let rectangle = Rectangle::new(width, height);
    let rectangle = rectangle.to_ring(1.0);
    let color = Color::srgba(255., 255., 0., 0.5);
    let xy: RegionXy = region.0.into();
    let x = xy.0.0 as f32 * width + width / 2.;
    let y = xy.0.1 as f32 * height + height / 2.;
    commands.spawn((
        RegionWireFrame(region.0.into()),
        RegionWireFrameDebug,
        Mesh2d(meshes.add(rectangle)),
        MeshMaterial2d(materials.add(color)),
        Transform::from_xyz(x, y, draw::Z_REGION_WIREFRAME),
    ));

    // World display
    let ratio = draw::world::ratio(window.size());
    let width = REGION_WIDTH as f32 * GEO_PIXELS_PER_TILE as f32 * ratio;
    let height = REGION_HEIGHT as f32 * GEO_PIXELS_PER_TILE as f32 * ratio;
    let rectangle = Rectangle::new(width, height);
    let rectangle = rectangle.to_ring(1.0);
    let color = Color::srgba(255., 255., 0., 0.5);
    let xy: RegionXy = region.0.into();
    let x = (xy.0.0 as f32 * width + width / 2.) + WORLD_MAP_X;
    let y = xy.0.1 as f32 * height + height / 2. + WORLD_MAP_Y;
    commands.spawn((
        RegionWireFrame(region.0.into()),
        RegionWireFrameDebug,
        Mesh2d(meshes.add(rectangle)),
        MeshMaterial2d(materials.add(color)),
        Transform::from_xyz(x, y, draw::Z_REGION_WIREFRAME),
    ));
}

pub fn on_forgotten_region(
    region: On<ForgottenRegion>,
    mut commands: Commands,
    query: Query<(Entity, &RegionWireFrame), With<RegionWireFrameDebug>>,
) {
    let region: RegionXy = region.0.into();
    for (entity, region_) in query {
        if region_.0 == region {
            commands.entity(entity).despawn();
        }
    }
}
