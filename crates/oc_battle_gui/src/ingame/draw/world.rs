use bevy::prelude::*;
use oc_root::{GEO_PIXELS_PER_TILE, WORLD_WIDTH};

pub fn ratio(window: Vec2) -> f32 {
    // TODO: choose adapt in width / height according to better choice to display entirely
    let ratio = window.x / (WORLD_WIDTH as f32 * GEO_PIXELS_PER_TILE as f32);
    ratio
}
