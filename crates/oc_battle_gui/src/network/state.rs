use bevy::prelude::*;

use crate::config::Connect;

#[derive(Debug, Default, Resource)]
pub struct State {
    pub server: Option<Connect>,
    pub connected: bool,
}
