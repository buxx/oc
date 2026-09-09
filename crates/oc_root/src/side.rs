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
#[cfg_attr(feature = "clap", derive(clap::ValueEnum,))]
#[rkyv(compare(PartialEq), derive(Debug))]
#[serde(rename_all = "kebab-case")]
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
