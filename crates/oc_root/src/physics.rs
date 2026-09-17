use std::ops::Deref;

use derive_more::{Add, Deref, Div, Mul, Sub};
use rkyv::Archive;

use crate::WorldConfig;

#[derive(
    Archive,
    rkyv::Deserialize,
    rkyv::Serialize,
    serde::Deserialize,
    serde::Serialize,
    Debug,
    PartialEq,
    Clone,
    Copy,
    Deref,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct Seconds(pub f32);

impl std::ops::Div<f32> for Seconds {
    type Output = Seconds;

    fn div(self, rhs: f32) -> Self::Output {
        Self(self.0 / rhs)
    }
}

impl std::ops::Mul<f32> for Seconds {
    type Output = Seconds;

    fn mul(self, rhs: f32) -> Self::Output {
        Self(self.0 * rhs)
    }
}

#[derive(
    Archive,
    rkyv::Deserialize,
    rkyv::Serialize,
    serde::Deserialize,
    serde::Serialize,
    Debug,
    PartialEq,
    PartialOrd,
    Clone,
    Copy,
    Default,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct Meters(pub f32);

impl Meters {
    pub const fn pixels(&self, w: &WorldConfig) -> f32 {
        self.0 * w.geo_pixels_per_meters
    }
}

impl std::ops::Mul<f32> for Meters {
    type Output = Meters;

    fn mul(self, rhs: f32) -> Self::Output {
        Self(self.0 * rhs)
    }
}

#[derive(
    Archive,
    rkyv::Deserialize,
    rkyv::Serialize,
    serde::Deserialize,
    serde::Serialize,
    Debug,
    PartialEq,
    Clone,
    Copy,
    PartialOrd,
    Add,
    Sub,
    Mul,
    Div,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct MetersSeconds(pub f32);

impl Deref for MetersSeconds {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
