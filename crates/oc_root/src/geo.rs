#[cfg(feature = "bevy")]
use bevy::math::Vec2 as BevyVec2;
use derive_more::{Constructor, From, Into};
use glam::Vec2;

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
pub struct WorldPoint2d {
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
pub struct WorldPoint3d {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// 2d coordinate translated on screen, where y axis is not fixed according to bevy y axis
#[derive(Debug, Clone, Copy)]
pub struct ScreenAwarePoint2d {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Constructor, From, Into)]
pub struct ScreenPoint2d {
    pub x: f32,
    pub y: f32,
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

impl_array_from!(WorldPoint2d, 2, [x, y]);
impl_array_from!(ScreenPoint2d, 2, [x, y]);
impl_array_from!(WorldPoint3d, 3, [x, y, z]);

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
impl WcfgFrom<BevyVec2> for WorldPoint2d {
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
impl From<BevyVec2> for ScreenPoint2d {
    fn from(value: BevyVec2) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}

impl Into<BevyVec2> for ScreenPoint2d {
    fn into(self) -> BevyVec2 {
        BevyVec2::new(self.x, self.y)
    }
}

// impl From<Vec2> for WorldPoint2d {
//     fn from(value: Vec2) -> Self {
//         Self {
//             x: value.x,
//             y: value.y,
//         }
//     }
// }

// impl From<Vec2> for ScreenAwarePoint2d {
//     fn from(value: Vec2) -> Self {
//         Self {
//             x: value.x,
//             y: value.y,
//         }
//     }
// }

// impl From<Vec2> for ScreenPoint2d {
//     fn from(value: Vec2) -> Self {
//         Self {
//             x: value.x,
//             y: value.y,
//         }
//     }
// }

impl WcfgFrom<WorldPoint2d> for ScreenPoint2d {
    fn from_(value: WorldPoint2d, w: &crate::WorldConfig) -> Self {
        Self {
            x: value.x,
            y: value.y.to_gui_y(w),
        }
    }
}
