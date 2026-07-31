#[cfg(feature = "bevy")]
use bevy::math::Vec2 as BevyVec2;
#[cfg(feature = "bevy")]
use bevy::math::Vec3 as BevyVec3;
use glam::{Vec2, Vec3};

use crate::WorldConfig;

// TODO: T not necessary ?
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

impl Y<Vec3> for Vec3 {
    fn to_world_y(&self, w: &WorldConfig) -> Vec3 {
        Vec3::new(self.x, w.world_height_pixels as f32 - self.y, self.z)
    }

    fn to_gui_y(&self, w: &WorldConfig) -> Vec3 {
        Vec3::new(self.x, w.world_height_pixels as f32 - self.y, self.z)
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

#[cfg(feature = "bevy")]
impl Y<BevyVec2> for BevyVec2 {
    fn to_world_y(&self, w: &WorldConfig) -> BevyVec2 {
        BevyVec2::new(self.x, w.world_height_pixels as f32 - self.y)
    }

    fn to_gui_y(&self, w: &WorldConfig) -> BevyVec2 {
        BevyVec2::new(self.x, w.world_height_pixels as f32 - self.y)
    }
}

#[cfg(feature = "bevy")]
impl Y<BevyVec3> for BevyVec3 {
    fn to_world_y(&self, w: &WorldConfig) -> BevyVec3 {
        BevyVec3::new(self.x, w.world_height_pixels as f32 - self.y, self.z)
    }

    fn to_gui_y(&self, w: &WorldConfig) -> BevyVec3 {
        BevyVec3::new(self.x, w.world_height_pixels as f32 - self.y, self.z)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum V {
    Server,
    Gui,
}
