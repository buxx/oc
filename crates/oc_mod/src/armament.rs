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
)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub enum ShotMode {
    Single,
    Burst3 { interval: f32 }, // seconds
}

impl ShotMode {
    pub fn name(&self) -> &str {
        match self {
            ShotMode::Single => "Single",
            ShotMode::Burst3 { interval: _ } => "Burst3",
        }
    }
}
