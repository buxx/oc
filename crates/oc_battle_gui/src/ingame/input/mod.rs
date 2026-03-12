use bevy::prelude::*;

pub mod client;
pub mod individual;
pub mod keyboard;
pub mod left_click;
pub mod map;
pub mod projectile;

#[derive(Debug, Resource, Default)]
pub struct State {
    pub clicks: Vec<Vec2>,
}
