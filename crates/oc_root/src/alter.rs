use std::str::FromStr;

use rkyv::Archive;
use strum::EnumCount;
use strum_macros::{EnumCount, EnumIter};

/// Visual alteration of a tile (like debris on the ground)
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
    EnumCount,
    EnumIter,
    serde::Deserialize,
    serde::Serialize,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub enum Alter {
    BrickDebris,
    MixedDebris,
    StoneDebris,
    WoodDebris,
    VegetalDebris,
}

impl Alter {
    pub const COUNT: usize = <Self as EnumCount>::COUNT;

    pub const fn index(self) -> usize {
        self as usize
    }
}

impl FromStr for Alter {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "BrickDebris" => Ok(Self::BrickDebris),
            "MixedDebris" => Ok(Self::MixedDebris),
            "StoneDebris" => Ok(Self::StoneDebris),
            "WoodDebris" => Ok(Self::WoodDebris),
            "VegetalDebris" => Ok(Self::VegetalDebris),
            _ => Err(format!("Unknown alter '{s}'")),
        }
    }
}
