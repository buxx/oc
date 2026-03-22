use std::ops::Deref;

use rkyv::{Archive, Deserialize, Serialize};

use crate::GEO_PIXELS_PER_METERS;

#[derive(Archive, Deserialize, Serialize, Debug, PartialEq, Clone)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct Meters(pub f32);

impl Meters {
    pub fn pixels(&self) -> f32 {
        self.0 * GEO_PIXELS_PER_METERS
    }
}

#[derive(Archive, Deserialize, Serialize, Debug, PartialEq, Clone)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct MetersSeconds(pub Meters);

impl Deref for MetersSeconds {
    type Target = Meters;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
