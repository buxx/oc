use bevy::prelude::*;

pub mod client;
pub mod individual;
pub mod keyboard;
#[cfg(feature = "debug")]
pub mod left_click;
pub mod projectile;

#[derive(Debug, Resource, Default)]
pub struct State {
    #[cfg(feature = "debug")]
    pub clicks: Vec<Vec2>,
}
