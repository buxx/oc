use std::ops::Deref;

use derive_more::Deref;
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
)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct Meters(pub f32);

impl Meters {
    pub const fn pixels(&self, w: &WorldConfig) -> f32 {
        self.0 * w.geo_pixels_per_meters
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
)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct MetersSeconds(pub f32);

impl Deref for MetersSeconds {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
