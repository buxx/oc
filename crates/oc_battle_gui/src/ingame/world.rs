use bevy::prelude::*;

use crate::{
    entity::world::VisibleBattleSquare,
    ingame::{
        camera::{map::world_map_point_to_bevy_world_point, move_::UpdateVisibleBattleSquare},
        draw,
    },
};

pub fn on_update_visible_battle_square(
    center: On<UpdateVisibleBattleSquare>,
    window: Single<&Window>,
    square: Single<(&mut Transform, &mut Mesh2d), With<VisibleBattleSquare>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let (mut transform, mut mesh) = square.into_inner();
    let point = world_map_point_to_bevy_world_point(center.0, window.size());
    let ratio = draw::world::ratio(window.size());

    transform.translation.x = point.x;
    transform.translation.y = point.y;

    let size = window.size() * ratio;
    mesh.0 = meshes.add(Rectangle::new(size.x, size.y).to_ring(1.0));
}
