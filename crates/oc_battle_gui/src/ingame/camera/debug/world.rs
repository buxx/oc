use bevy::prelude::*;

use crate::ingame::{
    camera::map::{
        WORLD_MAP_X, WORLD_MAP_Y, window_cursor_to_world_map_point,
        world_map_point_to_bevy_world_point,
    },
    draw,
};

#[derive(Debug, Component)]
pub struct WorldCursor;

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        WorldCursor,
        Transform::from_xyz(WORLD_MAP_X, WORLD_MAP_Y, draw::Z_WORLD_CURSOR),
        Mesh2d(meshes.add(Circle::new(4.0))),
        MeshMaterial2d(materials.add(Color::srgb(0., 0., 255.))),
    ));
}

pub fn cursor(window: Single<&Window>, mut cursor: Single<&mut Transform, With<WorldCursor>>) {
    let Some(point) = window.cursor_position() else {
        return;
    };

    // We do the compute in way then in opposite way to test code
    let point = window_cursor_to_world_map_point(point, window.size());
    let point = world_map_point_to_bevy_world_point(point, window.size());
    // dbg!(point);
    // let ratio = draw::world::ratio(window.size());
    // let point = point * ratio;
    // dbg!(point);
    cursor.translation.x = point.x;
    cursor.translation.y = point.y;
}
