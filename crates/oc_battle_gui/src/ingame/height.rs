use std::f32::consts::PI;
use std::path::PathBuf;

use bevy::color::palettes::css::WHITE;
use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll};
use bevy::prelude::*;
use bevy_heightmap::{HeightMap, HeightMapPlugin, ValueFunctionHeightMap};
use image::{GenericImage, ImageBuffer};
use oc_geo::region::{RegionXy, WorldRegionIndex};
use oc_geo::tile::{TileXy, WorldTileIndex};
use oc_root::{Wcfg, WcfgFrom, WcfgInto};
use oc_utils::d2::Xy;

use crate::states::{AppState, InGameState, Meta};
use crate::world::World;

pub struct HeightPlugin;

#[derive(Event)]
pub struct Spawn(pub Vec2);

#[derive(Component, Default)]
struct Terrain {}

const SCALE: f32 = 1024.;
const HEIGHT: f32 = 32.;
const THETA: f32 = PI / 8.;
const FOV: f32 = PI / 4.;

pub fn y_offset(z: f32) -> f32 {
    THETA.tan() * z
}

impl Plugin for HeightPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(HeightMapPlugin)
            .init_resource::<CameraOrbit>()
            .add_observer(on_spawn)
            .add_systems(
                Update,
                camera_control
                    .run_if(in_state(AppState::InGame))
                    .run_if(in_state(InGameState::Height)),
            );
    }
}

/// Tracks the orbit camera state so we can recompute the transform each frame.
#[derive(Resource)]
struct CameraOrbit {
    /// Horizontal angle around Z axis (yaw), in radians.
    yaw: f32,
    /// Vertical angle (pitch), in radians. Clamped so the camera stays above the terrain.
    pitch: f32,
    /// Distance from the focus point.
    distance: f32,
    /// World-space point the camera looks at.
    focus: Vec3,
}

impl Default for CameraOrbit {
    fn default() -> Self {
        Self {
            yaw: 0.0,
            pitch: PI / 2.0, // 90° — camera looks straight down
            distance: 1500.0,
            focus: Vec3::ZERO,
        }
    }
}

/// Recompute the camera `Transform` from the current `CameraOrbit` state.
fn orbit_transform(orbit: &CameraOrbit) -> Transform {
    // Spherical -> Cartesian offset from focus point.
    let offset = Vec3::new(
        orbit.distance * orbit.pitch.cos() * orbit.yaw.sin(),
        -orbit.distance * orbit.pitch.cos() * orbit.yaw.cos(),
        orbit.distance * orbit.pitch.sin(),
    );
    let eye = orbit.focus + offset;
    Transform::from_translation(eye).looking_at(orbit.focus, Vec3::Z)
}

/// AI generated function
fn camera_control(
    mut orbit: ResMut<CameraOrbit>,
    mut camera: Single<&mut Transform, With<Camera3d>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    mouse_scroll: Res<AccumulatedMouseScroll>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    let delta = mouse_motion.delta;

    let panning = mouse_buttons.pressed(MouseButton::Right)
        || (mouse_buttons.pressed(MouseButton::Left) && keyboard.pressed(KeyCode::ShiftLeft));
    let orbiting = mouse_buttons.pressed(MouseButton::Left) && !panning;

    if panning && delta != Vec2::ZERO {
        // Pan in the camera's local XY plane, scaled by distance so speed feels consistent.
        let pan_speed = orbit.distance * 0.0005;
        let transform = orbit_transform(&orbit);
        let right = transform.rotation * Vec3::X;
        let up = transform.rotation * Vec3::Y;
        orbit.focus -= right * delta.x * pan_speed;
        orbit.focus += up * delta.y * pan_speed;
    } else if orbiting && delta != Vec2::ZERO {
        let orbit_speed = 0.005;
        orbit.yaw -= delta.x * orbit_speed;
        orbit.pitch += delta.y * orbit_speed;
        // Keep pitch in a sane range: just above horizon to nearly straight down.
        orbit.pitch = orbit.pitch.clamp(0.05, PI / 2.0 - 0.01);
    }

    // Scroll to zoom. AccumulatedMouseScroll already normalises line vs. pixel units.
    let scroll = mouse_scroll.delta.y;
    if scroll != 0.0 {
        orbit.distance -= scroll * orbit.distance * 0.05;
        orbit.distance = orbit.distance.clamp(50.0, 4000.0);
    }

    **camera = orbit_transform(&orbit);
}

// FIXME: avoid blocking way of background + mesh generation by doing it in background and before real need
fn on_spawn(
    center: On<Spawn>,
    w: Res<Wcfg>,
    world: Res<World>,
    meta: Res<Meta>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
) {
    tracing::debug!("Spawn height map");
    let Some(w) = &w.0 else { return };
    let Some(meta) = &meta.0 else { return };
    let center = center.0;
    tracing::trace!(name="ingame-height-on-spawn-center", center=?center);

    let regions: Vec<WorldRegionIndex> = world.tiles.keys().cloned().collect();
    // FIXME BS NOW: compute size
    let regions_size = {
        let region_xys: Vec<RegionXy> =
            world.tiles.keys().map(|i| RegionXy::from_(*i, w)).collect();

        let min_x = region_xys.iter().map(|xy| xy.0.0).min();
        let max_x = region_xys.iter().map(|xy| xy.0.0).max();
        let min_y = region_xys.iter().map(|xy| xy.0.1).min();
        let max_y = region_xys.iter().map(|xy| xy.0.1).max();

        match (min_x, max_x, min_y, max_y) {
            (Some(min_x), Some(max_x), Some(min_y), Some(max_y)) => {
                Some((max_x - min_x, max_y - min_y))
            }
            _ => None,
        }
    };

    let (regions_width, regions_height) = match regions_size {
        Some((regions_width, regions_height)) => (regions_width + 1, regions_height + 1),
        None => return,
    };
    let (tiles_width, tiles_height) = (
        regions_width * w.region_width,
        regions_height * w.region_height,
    );
    let (background_width, background_height) = (
        tiles_width * w.geo_pixels_per_tile,
        tiles_height * w.geo_pixels_per_tile,
    );
    let (world_width, world_height) = (w.world_width_pixels, w.world_height_pixels);
    let tiles = &world.tiles;
    let tiles_size = UVec2::new(tiles_width as u32, tiles_height as u32);

    // FIXME paths ...
    let background = regions
        .iter()
        .map(|i| i.0.to_string())
        .collect::<Vec<String>>()
        .join("_");
    let background = PathBuf::from("cache_")
        .join("worlds")
        .join(meta.canonical())
        .join(format!("background_{}.png", background));

    tracing::trace!(
        name = "ingame-height-on-spawn-values",
        tiles_width = tiles_width,
        tiles_height = tiles_height,
        background_width = background_width,
        background_height = background_height,
        world_width = world_width,
        world_height = world_height,
        regions = ?regions,
        tiles_size = ?tiles_size,
        background = background.display().to_string(),
    );

    // FIXME refact ...
    // FIXME must be done in background task when move on battle or world map
    match std::fs::exists(PathBuf::from("assets").join(&background)).unwrap() {
        true => {
            tracing::trace!(name = "ingame-height-on-spawn-background-already-exist");
        }
        false => {
            tracing::trace!(name = "ingame-height-on-spawn-background-generate");
            let mut canvas: ImageBuffer<image::Rgba<u8>, Vec<u8>> = ImageBuffer::from_pixel(
                background_width as u32,
                background_height as u32,
                image::Rgba([0, 0, 0, 255]),
            );

            for region in regions {
                let xy: RegionXy = region.into_(w);
                let xy: TileXy = xy.into_(w);
                let (x, y) = (
                    xy.0.0 * w.geo_pixels_per_tile,
                    xy.0.1 * w.geo_pixels_per_tile,
                );
                // FIXME paths ...
                // FIXME: test if all images exists before, to retry later
                let region = PathBuf::from("assets")
                    .join("cache_")
                    .join("worlds")
                    .join(meta.canonical())
                    .join(format!("region{}.png", region.0));
                tracing::trace!(
                    name = "ingame-height-on-spawn-background-use",
                    region = region.display().to_string()
                );
                let region = image::open(&region).unwrap();

                canvas
                    .copy_from(&region.to_rgba8(), x as u32, y as u32)
                    .unwrap();
            }

            let path = PathBuf::from("assets").join(&background);
            tracing::trace!(
                name = "ingame-height-on-spawn-background-write",
                path = path.display().to_string()
            );
            canvas.save(&path).unwrap();
        }
    };

    let texture: Handle<Image> = asset_server.load(&background.display().to_string());
    tracing::trace!(name = "ingame-height-on-spawn-mesh-generate");
    let mesh: Handle<Mesh> = meshes.add(
        ValueFunctionHeightMap(|p: Vec2| {
            let p_ = Vec2::new(
                (p.x + 0.5) * world_width as f32,
                (p.y + 0.5) * world_height as f32,
            );
            let (x, y) = (p_.x as u64, (world_height as f32 - p_.y) as u64);
            let (x, y) = (x / w.geo_pixels_per_tile, y / w.geo_pixels_per_tile);
            let tile = TileXy(Xy(x, y));
            let tile_i: WorldTileIndex = tile.into_(w);
            let region_i: WorldRegionIndex = tile.into_(w);

            tiles
                .get(&region_i)
                .map(|tiles| tiles.get(&tile_i).map(|tile| tile.z as f32 * 0.1))
                .flatten()
                .unwrap_or_default()
        })
        .build_mesh(tiles_size),
    );

    tracing::trace!(name = "ingame-height-on-spawn-spawn");
    commands.spawn((
        Name::new("Terrain"),
        Terrain::default(),
        Mesh3d(mesh),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::WHITE,
            base_color_texture: Some(texture),
            perceptual_roughness: 1.0,  // fully rough = no gloss
            metallic: 0.0,              // no metallic sheen
            reflectance: 0.0,           // no specular reflection
            specular_transmission: 0.0, // no light transmission
            ..default()
        })),
        Transform {
            scale: Vec2::splat(SCALE).extend(HEIGHT),
            ..default()
        },
    ));
}

pub fn setup_camera(commands: &mut Commands) {
    let orbit = CameraOrbit::default();
    let transform = orbit_transform(&orbit);

    commands.spawn((
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: FOV,
            near: 0.1,
            far: 2000.,
            ..default()
        }),
        Transform::from_xyz(0.0, 200.0, 0.0) // directly above
            .looking_at(Vec3::ZERO, Vec3::NEG_Z), // look down, -Z as "up" on screen
    ));

    commands.spawn((
        Transform::from_xyz(0.0, 0.0, orbit.distance) // original light position
            .with_rotation(Quat::from_axis_angle(Vec3::ONE, -PI / 6.)),
        DirectionalLight {
            color: WHITE.into(),
            illuminance: 4500.,
            shadows_enabled: true,
            ..default()
        },
    ));
}
