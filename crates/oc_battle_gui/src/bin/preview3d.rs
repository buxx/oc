use std::f32::consts::PI;
use std::path::PathBuf;

use bevy::{
    color::palettes::css::WHITE,
    input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll},
    prelude::*,
};
use bevy_heightmap::{HeightMap, HeightMapPlugin, ValueFunctionHeightMap};
use clap::Parser;
use oc_root::WorldConfig;
use oc_world::reader::MapReader;

pub const SCALE: f32 = 1024.;
pub const HEIGHT: f32 = 32.;
pub const THETA: f32 = PI / 8.;
pub const FOV: f32 = PI / 4.;
pub fn y_offset(z: f32) -> f32 {
    THETA.tan() * z
}

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
struct Args {
    #[clap()]
    world: PathBuf,
}

#[derive(Event)]
struct Spawn(PathBuf);

#[derive(Component, Default)]
struct Terrain {}

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
            pitch: THETA,
            distance: 1500.0,
            focus: Vec3::ZERO,
        }
    }
}

fn main() {
    let args = Args::parse();

    let init = move |mut commands: Commands| {
        commands.trigger(Spawn(args.world.clone()));
    };

    let mut app = App::new();
    app.add_plugins((DefaultPlugins, HeightMapPlugin))
        .init_resource::<CameraOrbit>()
        .add_observer(on_spawn)
        .add_systems(Startup, (setup, init))
        .add_systems(Update, camera_control)
        .run();
}

fn setup(mut commands: Commands) {
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
        transform,
    ));
    commands.spawn((
        Transform::from_xyz(0.0, 0.0, orbit.distance)
            .with_rotation(Quat::from_axis_angle(Vec3::ONE, -PI / 6.)),
        DirectionalLight {
            color: WHITE.into(),
            illuminance: 4500.,
            shadows_enabled: true,
            ..default()
        },
    ));
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

fn camera_control(
    mut orbit: ResMut<CameraOrbit>,
    mut camera: Single<&mut Transform, With<Camera3d>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    // Bevy 0.17+: accumulated resources replace EventReader<MouseMotion/MouseWheel>
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

fn on_spawn(
    spawn: On<Spawn>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
) {
    let reader = MapReader::new(&spawn.0).unwrap();
    let (width, height) = (reader.width().unwrap(), reader.height().unwrap());
    let w = WorldConfig::new(width as u64, height as u64);
    let tiles = reader.tiles(&w).unwrap();
    let size = UVec2::new(width, height);
    // TODO: degeu
    let background = PathBuf::from("worlds_")
        .join(spawn.0.iter().last().unwrap())
        .join("background.png");
    let texture: Handle<Image> = asset_server.load(background);
    let mesh: Handle<Mesh> = meshes.add(
        ValueFunctionHeightMap(|p: Vec2| {
            let p_ = Vec2::new((p.x + 0.5) * width as f32, (p.y + 0.5) * height as f32);
            let (x, y) = (p_.x as usize, (height as f32 - p_.y) as usize);

            if y == height as usize || x == width as usize {
                return 0.0;
            }

            let i = y * width as usize + x;
            tiles[i].z as f32 * 0.1
        })
        .build_mesh(size),
    );
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
