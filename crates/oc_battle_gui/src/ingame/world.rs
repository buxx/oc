use bevy::prelude::*;

use crate::{
    entity::world::VisibleBattleSquare,
    ingame::camera::{
        map::{WORLD_MAP_X, WORLD_MAP_Y},
        move_::UpdateVisibleBattleSquare,
    },
};

pub fn on_update_visible_battle_square(
    _: On<UpdateVisibleBattleSquare>,
    square: Single<(&mut Transform, &mut Mesh2d), With<VisibleBattleSquare>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let (mut transform, mut mesh) = square.into_inner();

    transform.translation.x = WORLD_MAP_X;
    transform.translation.y = WORLD_MAP_Y;
    mesh.0 = meshes.add(Rectangle::new(50.0, 50.0).to_ring(1.0));
}
