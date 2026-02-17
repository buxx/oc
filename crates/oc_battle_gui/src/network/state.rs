use std::net::SocketAddr;

use bevy::prelude::*;

#[derive(Debug, Default, Resource)]
pub struct State {
    pub connected: Option<SocketAddr>,
}
