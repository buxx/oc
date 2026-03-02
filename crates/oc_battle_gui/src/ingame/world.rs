use std::path::PathBuf;

use bevy::prelude::*;
use oc_root::{MINIMAP_HEIGHT_PIXELS, MINIMAP_WIDTH_PIXELS};

use crate::{
    entity::world::{VisibleBattleSquare, WorldMapBackground, minimap::Minimap},
    ingame::{
        camera::{map::world_map_point_to_bevy_world_point, move_::UpdateVisibleBattleSquare},
        draw::{
            Z_WORLD_MAP_BACKGROUND, Z_WORLD_MAP_BATTLE_SQUARE, Z_WORLD_MAP_MINIMAP,
            world::WorldMapDisplay,
        },
    },
    states::Meta,
};

#[derive(Debug, Event)]
pub struct SpawnMinimap;

#[derive(Debug, Event)]
pub struct SpawnVisibleBattleSquare;

#[derive(Debug, Event)]
pub struct SpawnWorldMapBackground;

#[derive(Debug, Event)]
pub struct DespawnWorldMapBackground;

pub fn on_spawn_minimap(
    _: On<SpawnMinimap>,
    mut commands: Commands,
    window: Single<&Window>,
    assets: Res<AssetServer>,
    meta: Res<Meta>,
) {
    let Some(meta) = &meta.0 else { return };

    let display = WorldMapDisplay::from_env(window.size());
    let path = PathBuf::from(".cache").join("maps");
    let path = path.join(meta.folder_name());
    let path = path.join("minimap.png");

    let x = display.center.x;
    let y = display.center.y;
    let scale_x = display.size.x / MINIMAP_WIDTH_PIXELS as f32;
    let scale_y = display.size.y / MINIMAP_HEIGHT_PIXELS as f32;
    commands.spawn((
        Minimap,
        Sprite::from_image(assets.load(path)),
        Transform {
            scale: Vec3::new(scale_x, -scale_y, 1.0), // Mirror on Y-axis
            translation: Vec3::new(x, y, Z_WORLD_MAP_MINIMAP),
            ..default()
        },
    ));
}
pub fn on_spawn_visible_battle_square(
    _: On<SpawnVisibleBattleSquare>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    window: Single<&Window>,
) {
    let display = WorldMapDisplay::from_env(window.size());

    commands.spawn((
        VisibleBattleSquare,
        Transform::from_xyz(display.start.x, display.start.y, Z_WORLD_MAP_BATTLE_SQUARE),
        Mesh2d(meshes.add(Rectangle::new(0.0, 0.0).to_ring(1.0))),
        MeshMaterial2d(materials.add(Color::srgb(0.8, 0.8, 0.8))),
    ));
}

pub fn on_spawn_world_map_background(
    _: On<SpawnWorldMapBackground>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    window: Single<&Window>,
) {
    let display = WorldMapDisplay::from_env(window.size());

    commands.spawn((
        WorldMapBackground,
        Transform::from_xyz(display.center.x, display.center.y, Z_WORLD_MAP_BACKGROUND),
        Mesh2d(meshes.add(Rectangle::new(display.size.x, display.size.y))),
        MeshMaterial2d(materials.add(Color::srgb(0.1, 0.1, 0.1))),
    ));
}

pub fn on_despawn_world_map_background(
    _: On<DespawnWorldMapBackground>,
    mut commands: Commands,
    query: Single<Entity, With<WorldMapBackground>>,
) {
    commands.entity(query.into_inner()).despawn();
}

pub fn on_update_battle_square(
    center: On<UpdateVisibleBattleSquare>,
    window: Single<&Window>,
    square: Single<(&mut Transform, &mut Mesh2d), With<VisibleBattleSquare>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let (mut transform, mut mesh) = square.into_inner();
    let display = WorldMapDisplay::from_env(window.size());
    let point = world_map_point_to_bevy_world_point(center.0, window.size());

    transform.translation.x = point.x;
    transform.translation.y = point.y;

    let size = window.size() * display.ratio;
    mesh.0 = meshes.add(Rectangle::new(size.x, size.y).to_ring(1.0));
}
