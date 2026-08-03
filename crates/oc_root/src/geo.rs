#[cfg(feature = "bevy")]
use bevy::math::Vec2 as BevyVec2;
#[cfg(feature = "bevy")]
use bevy::math::Vec3 as BevyVec3;
use derive_more::{Constructor, From, Into};
use glam::Vec2;
use glam::Vec3;

use crate::{WcfgFrom, y::Y};

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    serde::Serialize,
    serde::Deserialize,
    Constructor,
    From,
    Into,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct WorldVec2 {
    pub x: f32,
    pub y: f32,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    serde::Serialize,
    serde::Deserialize,
    Constructor,
    From,
    Into,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct WorldVec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

// /// 3D point in context which can be used also in world or in screen
// #[derive(Debug, Clone, Copy, PartialEq, Constructor, From, Into)]
// pub struct AwareVec3 {
//     pub x: f32,
//     pub y: f32,
//     pub z: f32,
// }

// /// 2d coordinate translated on screen, where y axis is not fixed according to bevy y axis
// #[derive(Debug, Clone, Copy)]
// pub struct ScreenAwarePoint2d {
//     pub x: f32,
//     pub y: f32,
// }

#[derive(Debug, Clone, Copy, PartialEq, Constructor, From, Into)]
pub struct ScreenVec2 {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Constructor, From, Into)]
pub struct ScreenVec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

macro_rules! impl_array_from {
    ($ty:ty, $n:literal, [$($field:ident),+]) => {
        impl From<[f32; $n]> for $ty {
            fn from([$($field),+]: [f32; $n]) -> Self {
                Self { $($field),+ }
            }
        }

        impl From<$ty> for [f32; $n] {
            fn from(p: $ty) -> Self {
                [$(p.$field),+]
            }
        }
    };
}

impl_array_from!(WorldVec2, 2, [x, y]);
impl_array_from!(ScreenVec2, 2, [x, y]);
impl_array_from!(WorldVec3, 3, [x, y, z]);

impl From<WorldVec3> for WorldVec2 {
    fn from(value: WorldVec3) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}

#[cfg(feature = "bevy")]
impl From<WorldVec3> for bevy::math::Vec3 {
    fn from(value: WorldVec3) -> Self {
        bevy::math::Vec3::new(value.x, value.y, value.z)
    }
}

// #[cfg(feature = "bevy")]
// impl From<BevyVec2> for WorldPoint2d {
//     fn from(value: BevyVec2) -> Self {
//         Self {
//             x: value.x,
//             y: value.y,
//         }
//     }
// }

#[cfg(feature = "bevy")]
impl WcfgFrom<BevyVec2> for WorldVec2 {
    fn from_(value: BevyVec2, w: &crate::WorldConfig) -> Self {
        Self {
            x: value.x,
            y: value.y.to_world_y(w),
        }
    }
}

// #[cfg(feature = "bevy")]
// impl From<BevyVec2> for ScreenAwarePoint2d {
//     fn from(value: BevyVec2) -> Self {
//         Self {
//             x: value.x,
//             y: value.y,
//         }
//     }
// }

#[cfg(feature = "bevy")]
impl From<BevyVec2> for ScreenVec2 {
    fn from(value: BevyVec2) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}

#[cfg(feature = "bevy")]
impl Into<BevyVec2> for ScreenVec2 {
    fn into(self) -> BevyVec2 {
        BevyVec2::new(self.x, self.y)
    }
}

#[cfg(feature = "bevy")]
impl Into<BevyVec3> for ScreenVec3 {
    fn into(self) -> BevyVec3 {
        BevyVec3::new(self.x, self.y, self.z)
    }
}

impl WcfgFrom<WorldVec2> for ScreenVec2 {
    fn from_(value: WorldVec2, w: &crate::WorldConfig) -> Self {
        Self {
            x: value.x,
            y: value.y.to_gui_y(w),
        }
    }
}

impl WcfgFrom<ScreenVec2> for WorldVec2 {
    fn from_(value: ScreenVec2, w: &crate::WorldConfig) -> Self {
        Self {
            x: value.x,
            y: value.y.to_world_y(w),
        }
    }
}

impl WcfgFrom<WorldVec3> for ScreenVec2 {
    fn from_(value: WorldVec3, w: &crate::WorldConfig) -> Self {
        Self {
            x: value.x,
            y: value.y.to_gui_y(w),
        }
    }
}

impl WcfgFrom<WorldVec3> for ScreenVec3 {
    fn from_(value: WorldVec3, w: &crate::WorldConfig) -> Self {
        Self {
            x: value.x,
            y: value.y.to_gui_y(w),
            z: value.z,
        }
    }
}

impl WorldVec2 {
    pub fn extend(&self, z: f32) -> WorldVec3 {
        WorldVec3::new(self.x, self.y, z)
    }
}

impl ScreenVec2 {
    pub fn extend(&self, z: f32) -> ScreenVec3 {
        ScreenVec3::new(self.x, self.y, z)
    }
}

impl WorldVec3 {
    #[inline]
    pub fn sub(self, rhs: Self) -> Self {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }

    #[inline]
    pub fn normalize_or_zero(&self) -> Self {
        let vec = Vec3::new(self.x, self.y, self.z).normalize_or_zero();
        Self::new(vec.x, vec.y, vec.z)
    }
}

impl std::ops::Sub for WorldVec3 {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self {
        WorldVec3::sub(self, rhs)
    }
}

impl std::ops::SubAssign for WorldVec3 {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

impl WorldVec2 {
    #[inline]
    pub fn sub(self, rhs: Self) -> Self {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }

    #[inline]
    pub fn normalize_or_zero(&self) -> Self {
        let vec = Vec2::new(self.x, self.y).normalize_or_zero();
        Self::new(vec.x, vec.y)
    }
}

impl std::ops::Sub for WorldVec2 {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self {
        WorldVec2::sub(self, rhs)
    }
}

impl std::ops::SubAssign for WorldVec2 {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}
