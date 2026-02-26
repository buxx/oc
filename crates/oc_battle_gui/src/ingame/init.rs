use bevy::prelude::*;
use oc_network::ToServer;

use crate::{
    entity::world::VisibleBattleSquare,
    ingame::draw::{
        self,
        world::{WORLD_MAP_X, WORLD_MAP_Y},
    },
    network::output::ToServerEvent,
};

pub fn refresh(mut commands: Commands) {
    commands.trigger(ToServerEvent(ToServer::Refresh.into()));
}

pub fn spawn_visible_battle_square(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        VisibleBattleSquare,
        Transform::from_xyz(WORLD_MAP_X, WORLD_MAP_Y, draw::Z_WORLD_BATTLE_SQUARE),
        Mesh2d(meshes.add(Rectangle::new(0.0, 0.0).to_ring(1.0))),
        MeshMaterial2d(materials.add(Color::srgb(200., 200., 200.))),
    ));
}
