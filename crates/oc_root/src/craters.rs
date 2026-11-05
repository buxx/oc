use rkyv::Archive;

/// Kind of crater left on the ground by an explosion, according to the ground hardness
#[derive(
    Debug,
    Clone,
    Copy,
    Archive,
    rkyv::Deserialize,
    rkyv::Serialize,
    PartialEq,
    Eq,
    Hash,
    serde::Deserialize,
    serde::Serialize,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub enum CraterType {
    Mud,
    Concrete,
}

impl CraterType {
    /// Row of this type in the sprite file
    pub const fn row(self) -> usize {
        self as usize
    }
}
