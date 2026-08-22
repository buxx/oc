use std::time::Instant;

use bevy::prelude::*;
use oc_network::Debug;

use crate::debug::DebugEvent;

pub mod projectile;

pub fn on_debug_input(event: On<DebugEvent>, mut state: ResMut<crate::ingame::state::State>) {
    match event.0 {
        Debug::Collision(position) => state.collisions.push((Instant::now(), position)),
    }
}
