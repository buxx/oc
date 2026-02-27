use bevy::prelude::*;

use crate::{
    entity::world::{VisibleBattleSquare, WorldMapBackground},
    ingame::{
        camera::{map::world_map_point_to_bevy_world_point, move_::UpdateVisibleBattleSquare},
        draw::{self, world::WorldMapDisplay},
    },
};

#[derive(Debug, Event)]
pub struct SpawnVisibleBattleSquare;

#[derive(Debug, Event)]
pub struct SpawnWorldMapBackground;

#[derive(Debug, Event)]
pub struct DespawnWorldMapBackground;

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
        Transform::from_xyz(display.start.x, display.start.y, display.z),
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
        Transform::from_xyz(display.center.x, display.center.y, display.z),
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
