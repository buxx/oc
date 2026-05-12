use std::f32::consts::PI;
use std::path::PathBuf;

use bevy::color::palettes::css::WHITE;
use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy_heightmap::{HeightMap, HeightMapPlugin, ValueFunctionHeightMap};
use oc_geo::region::{RegionXy, WorldRegionIndex};
use oc_geo::tile::{TileXy, WorldTileIndex};
use oc_root::y::Y;
use oc_root::{Wcfg, WcfgInto};
use oc_utils::d2::Xy;

use crate::states::{AppState, InGameState, Meta};
use crate::world::World;

pub struct HeightPlugin;

#[derive(Event)]
pub struct Spawn(pub Vec2);

#[derive(Component, Default)]
struct Height {}

impl Plugin for HeightPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(HeightMapPlugin)
            .add_observer(on_spawn)
            .add_systems(
                Update,
                (
                    move_camera_by_keyboard,
                    move_camera_by_mouse,
                    rotate_camera_by_mouse,
                )
                    .run_if(in_state(AppState::InGame))
                    .run_if(in_state(InGameState::Height)),
            );
    }
}

fn rotate_camera_by_mouse(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: MessageReader<MouseMotion>,
    mut query: Query<&mut Transform, With<Camera3d>>,
) {
    let Ok(mut transform) = query.single_mut() else {
        return;
    };

    if mouse_buttons.pressed(MouseButton::Left) {
        for event in mouse_motion.read() {
            let yaw = -event.delta.x * 0.002;
            let pitch = -event.delta.y * 0.002;

            // The point the camera is looking at (world space)
            let target =
                transform.translation + transform.forward() * transform.translation.length();

            // Orbit around target
            let yaw_quat = Quat::from_rotation_y(yaw);
            let pitch_quat = Quat::from_axis_angle(*transform.right(), pitch);

            // Rotate position around target
            let offset = transform.translation - target;
            let new_offset = yaw_quat * pitch_quat * offset;
            transform.translation = target + new_offset;

            // Always look back at target
            transform.look_at(target, Vec3::Y);
        }
    } else {
        mouse_motion.clear();
    }
}

fn move_camera_by_mouse(
    mut dragging: Local<bool>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: MessageReader<MouseMotion>,
    mut query: Query<(&mut Transform, &Projection), With<Camera3d>>,
) {
    let Ok((mut transform, projection)) = query.single_mut() else {
        return;
    };

    let scale = match projection {
        Projection::Orthographic(o) => o.scale,
        _ => 1.0,
    };

    if mouse_buttons.just_pressed(MouseButton::Right) {
        *dragging = true;
    }
    if mouse_buttons.just_released(MouseButton::Right) {
        *dragging = false;
    }

    if *dragging {
        for event in mouse_motion.read() {
            transform.translation.x -= event.delta.x * scale;
            transform.translation.y += event.delta.y * scale;
        }
    } else {
        mouse_motion.clear();
    }
}

const CAMERA_SPEED: f32 = 300.0; // units per second

fn move_camera_by_keyboard(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<Camera3d>>,
) {
    let Ok(mut transform) = query.single_mut() else {
        return;
    };

    let mut direction = Vec3::ZERO;

    if keys.pressed(KeyCode::ArrowLeft) || keys.pressed(KeyCode::KeyA) {
        direction.x -= 1.0;
    }
    if keys.pressed(KeyCode::ArrowRight) || keys.pressed(KeyCode::KeyD) {
        direction.x += 1.0;
    }
    if keys.pressed(KeyCode::ArrowUp) || keys.pressed(KeyCode::KeyW) {
        direction.y += 1.0;
    }
    if keys.pressed(KeyCode::ArrowDown) || keys.pressed(KeyCode::KeyS) {
        direction.y -= 1.0;
    }

    // Normalize to prevent faster diagonal movement
    if direction != Vec3::ZERO {
        direction = direction.normalize();
    }

    transform.translation += direction * CAMERA_SPEED * time.delta_secs();
    // dbg!(transform.translation);
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
    let grid_size = UVec2::new(w.region_width as u32, w.region_height as u32);

    for region in regions {
        // FIXME Files
        let texture = PathBuf::from("cache_")
            .join("worlds")
            .join(meta.canonical())
            .join(format!("region{}.png", region.0));
        if !std::fs::exists(PathBuf::from("assets").join(&texture)).unwrap() {
            tracing::warn!(
                "Can't build heigh map for region {region:?}: background file {} do not exist",
                texture.display()
            );
            continue;
        }
        let texture: Handle<Image> = asset_server.load(&texture.display().to_string());
        tracing::trace!(name = "ingame-height-on-spawn-mesh-generate", region = ?region);

        let Some(tiles) = world.tiles.get(&region) else {
            tracing::warn!("Can't build heigh map for region {region:?}: no known tiles",);
            continue;
        };
        let mesh: Handle<Mesh> = meshes.add(
            ValueFunctionHeightMap(|p: Vec2| {
                // p is given as retlative like (top-left) -0.5,-0.5, (center) 0.0,0.0, etc.
                // So, add 0.5 to have something relative from 0.0 to 1.0,
                // then, * region_width/ieght to find point is.
                let p_ = Vec2::new(
                    (p.x + 0.5) * w.region_width as f32,
                    (p.y + 0.5) * w.region_height as f32,
                );
                // Remove region_height to adapt to inverted y
                let (x, y) = (p_.x as u64, (w.region_height as f32 - p_.y) as u64);
                let tile = TileXy(Xy(x, y));
                let tile_i: WorldTileIndex = tile.into_(w);

                tiles
                    .get(&tile_i)
                    .map(|tile| tile.z as f32 * 0.1) // FIXME BS NOW
                    .unwrap_or_default()
            })
            .build_mesh(grid_size),
        );

        tracing::trace!(name = "ingame-height-on-spawn-spawn");

        let width = w.region_width_pixels as f32;
        let height = w.region_height_pixels as f32;

        let region: RegionXy = region.into_(w);
        let x = region.0.0 as f32 * width;
        let y = region.0.1 as f32 * height;
        let x = x + width / 2.;
        let y = y + height / 2.;
        let x = x;
        let y = y.to_gui_y(w);

        commands.spawn((
            Height::default(),
            Mesh3d(mesh),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::WHITE,
                base_color_texture: Some(texture),
                perceptual_roughness: 1.0,
                metallic: 0.0,
                reflectance: 0.0,
                specular_transmission: 0.0,
                ..default()
            })),
            Transform {
                translation: Vec3::new(x, y, 0.),
                scale: Vec2::new(width, height).extend(24.0),
                ..default()
            },
        ));
    }
}

pub fn setup_camera3d(commands: &mut Commands, center: &Vec2) {
    let mut transform = Transform::from_xyz(0.0, 0.0, 1000.0).looking_at(Vec3::ZERO, Vec3::Y);
    transform.translation.x = center.x;
    transform.translation.y = center.y;

    commands.spawn((
        Camera3d::default(),
        Projection::Orthographic(OrthographicProjection {
            scale: 1.0,
            near: -100.0, // negative near lets you see meshes slightly in front of camera Z
            far: 2000.0,
            ..OrthographicProjection::default_3d()
        }),
        transform,
    ));
}

pub fn setup_light3d(commands: &mut Commands) {
    commands.spawn((
        Transform::from_xyz(0.0, 0.0, 1000.) // original light position
            .with_rotation(Quat::from_axis_angle(Vec3::ONE, -PI / 6.)),
        DirectionalLight {
            color: WHITE.into(),
            illuminance: 4500.,
            shadows_enabled: true,
            ..default()
        },
    ));
}
