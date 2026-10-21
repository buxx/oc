use std::time::Instant;

use bevy::prelude::*;
use oc_network::Debug;

use crate::debug::DebugEvent;

pub mod behavior;
pub mod projectile;
pub mod visibility;

pub fn on_debug_input(event: On<DebugEvent>, mut state: ResMut<crate::ingame::state::State>) {
    match event.0 {
        Debug::Collision(position, lag) => state.collisions.push((Instant::now() + lag, position)),
    }
}
