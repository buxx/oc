use glam::Vec2;

use crate::WorldConfig;

pub trait Y<T> {
    fn to_world_y(&self, w: &WorldConfig) -> T;
    fn to_gui_y(&self, w: &WorldConfig) -> T;
}

impl Y<f32> for f32 {
    fn to_world_y(&self, w: &WorldConfig) -> f32 {
        w.world_height_pixels as f32 - self
    }

    fn to_gui_y(&self, w: &WorldConfig) -> f32 {
        w.world_height_pixels as f32 - self
    }
}

impl Y<Vec2> for Vec2 {
    fn to_world_y(&self, w: &WorldConfig) -> Vec2 {
        Vec2::new(self.x, w.world_height_pixels as f32 - self.y)
    }

    fn to_gui_y(&self, w: &WorldConfig) -> Vec2 {
        Vec2::new(self.x, w.world_height_pixels as f32 - self.y)
    }
}

impl Y<[f32; 3]> for [f32; 3] {
    fn to_world_y(&self, w: &WorldConfig) -> [f32; 3] {
        [self[0], w.world_height_pixels as f32 - self[1], self[2]]
    }

    fn to_gui_y(&self, w: &WorldConfig) -> [f32; 3] {
        [self[0], w.world_height_pixels as f32 - self[1], self[2]]
    }
}
