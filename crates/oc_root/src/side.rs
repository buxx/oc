use rkyv::Archive;

#[derive(
    Debug,
    Clone,
    Copy,
    Archive,
    rkyv::Deserialize,
    rkyv::Serialize,
    serde::Serialize,
    serde::Deserialize,
    PartialEq,
    Eq,
    Hash,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub enum Side {
    A,
    B,
}
impl Side {
    pub fn opposite(&self) -> Side {
        match self {
            Side::A => Side::B,
            Side::B => Side::A,
        }
    }

    pub fn letter(&self) -> char {
        match self {
            Side::A => 'A',
            Side::B => 'B',
        }
    }
}
