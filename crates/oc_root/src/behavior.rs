use rkyv::Archive;

#[derive(
    Debug,
    Clone,
    Copy,
    Archive,
    rkyv::Deserialize,
    rkyv::Serialize,
    PartialEq,
    serde::Deserialize,
    serde::Serialize,
    Eq,
    PartialOrd,
    Ord,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct Suppress(pub u8);

impl Suppress {
    pub fn zero() -> Self {
        Self(0)
    }

    pub fn normalize(&self) -> f32 {
        self.0 as f32 / 255.0
    }

    pub fn decrease(mut self, value: Suppress) -> Suppress {
        self.0 = self.0.saturating_sub(value.0);
        self
    }
}
