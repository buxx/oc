use bevy::prelude::*;
use oc_network::ToServer;

// #[cfg(feature = "debug")]
// use oc_physics::{Force, MetersSeconds};
// #[cfg(feature = "debug")]
// use oc_projectile::{Projectile, bullet::Bullet};

use crate::{
    ingame::{
        FirstIngameEnter,
        world::{SpawnMinimap, SpawnVisibleBattleSquare, SpawnWorldMapBackground},
    },
    network::output::ToServerEvent,
};

pub fn init(mut commands: Commands) {
    commands.trigger(FirstIngameEnter);
}

pub fn refresh(mut commands: Commands) {
    commands.trigger(ToServerEvent(ToServer::Refresh.into()));
}

pub fn spawn_world_map(mut commands: Commands) {
    commands.trigger(SpawnMinimap);
    commands.trigger(SpawnWorldMapBackground);
    commands.trigger(SpawnVisibleBattleSquare);
}

// #[cfg(feature = "debug")]
// pub fn on_first_ingame_enter(_: On<FirstIngameEnter>, mut commands: Commands) {
//     let position = [0., 0.];
//     let direction = [0.5, 0.5];
//     let speed = MetersSeconds(20.);
//     let thrust = Force::Translation(direction, speed);
//     let forces = vec![thrust];
//     let bullet = Bullet::new(position, forces);
//     let projectile = Projectile::Bullet(bullet);
//     let projectile = SpawnProjectile(projectile);

//     tracing::debug!("Spawn projectile !");
//     commands.trigger(ToServerEvent(ToServer::SpawnProjectile(projectile)));
// }
