use derive_more::{Add, Deref};
use rkyv::{Archive, Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deref, PartialEq, Archive, Deserialize, Serialize)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct Opacity(pub f32);

#[derive(
    Debug, Clone, Copy, Deref, PartialEq, PartialOrd, Add, Archive, Deserialize, Serialize,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct CumulatedOpacity(pub f32);

impl CumulatedOpacity {
    pub fn inaccuracy(&self, w: &crate::WorldConfig) -> f32 {
        self.0 * w.opacity_inaccuracy
    }
}
