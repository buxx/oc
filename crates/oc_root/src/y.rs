#[cfg(feature = "bevy")]
use bevy;

use crate::WORLD_HEIGHT_PIXELS;
use glam::Vec2;

pub trait Y<T> {
    fn to_world_y(&self) -> T;
    fn to_gui_y(&self) -> T;
}

impl Y<f32> for f32 {
    fn to_world_y(&self) -> f32 {
        WORLD_HEIGHT_PIXELS as f32 - self
    }

    fn to_gui_y(&self) -> f32 {
        WORLD_HEIGHT_PIXELS as f32 - self
    }
}

impl Y<Vec2> for Vec2 {
    fn to_world_y(&self) -> Vec2 {
        Vec2::new(self.x, WORLD_HEIGHT_PIXELS as f32 - self.y)
    }

    fn to_gui_y(&self) -> Vec2 {
        Vec2::new(self.x, WORLD_HEIGHT_PIXELS as f32 - self.y)
    }
}

#[cfg(feature = "bevy")]
impl Y<bevy::math::Vec2> for bevy::math::Vec2 {
    fn to_world_y(&self) -> bevy::math::Vec2 {
        bevy::math::Vec2::new(self.x, WORLD_HEIGHT_PIXELS as f32 - self.y)
    }

    fn to_gui_y(&self) -> bevy::math::Vec2 {
        bevy::math::Vec2::new(self.x, WORLD_HEIGHT_PIXELS as f32 - self.y)
    }
}

impl Y<[f32; 3]> for [f32; 3] {
    fn to_world_y(&self) -> [f32; 3] {
        [self[0], WORLD_HEIGHT_PIXELS as f32 - self[1], self[2]]
    }

    fn to_gui_y(&self) -> [f32; 3] {
        [self[0], WORLD_HEIGHT_PIXELS as f32 - self[1], self[2]]
    }
}
