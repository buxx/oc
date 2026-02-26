use bevy::prelude::*;
use oc_root::{GEO_PIXELS_PER_TILE, WORLD_HEIGHT, WORLD_WIDTH};

pub const WORLD_MAP_X: f32 = -100_000.;
pub const WORLD_MAP_Y: f32 = -100_000.;

pub fn ratio(window: Vec2) -> Vec2 {
    // TODO: choose adapt in width / height according to better choice to display entirely
    let ratio_x = window.x / (WORLD_WIDTH as f32 * GEO_PIXELS_PER_TILE as f32);
    let ratio_y = window.y / (WORLD_HEIGHT as f32 * GEO_PIXELS_PER_TILE as f32);

    Vec2::new(ratio_x, ratio_y)
}
