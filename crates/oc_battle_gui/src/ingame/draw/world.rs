use bevy::prelude::*;
use oc_root::{WORLD_HEIGHT_PIXELS, WORLD_WIDTH_PIXELS};

const WORLD_MAP_OFFSET_X: f32 = -100_000.;
const WORLD_MAP_OFFSET_Y: f32 = -100_000.;
const WORLD_MAP_WINDOW_PADDING: f32 = 10.0;

pub struct WorldMapDisplay {
    pub start: Vec2,
    pub size: Vec2,
    pub center: Vec2,
    pub z: f32,
    pub padding: Vec2,
    pub ratio: Vec2,
}

impl WorldMapDisplay {
    pub fn from_env(window: Vec2) -> Self {
        let start = Vec2::new(WORLD_MAP_OFFSET_X, WORLD_MAP_OFFSET_Y) + WORLD_MAP_WINDOW_PADDING;
        let size = window - (WORLD_MAP_WINDOW_PADDING * 2.);
        let center = start + (size / 2.);
        let z = super::Z_WORLD_MAP_BACKGROUND;
        let padding = Vec2::new(WORLD_MAP_WINDOW_PADDING, -WORLD_MAP_WINDOW_PADDING);
        let world = Vec2::new(WORLD_WIDTH_PIXELS as f32, WORLD_HEIGHT_PIXELS as f32);
        let ratio = size / world;

        Self {
            start,
            size,
            center,
            z,
            padding,
            ratio,
        }
    }
}
