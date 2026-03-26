use derive_more::Deref;
use oc_root::physics::Seconds;
use rkyv::Archive;

use crate::{Mod, sound::SoundIndex};

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
    Deref,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct ShotModeIndex(pub u32);

#[derive(
    Debug,
    Clone,
    Archive,
    rkyv::Deserialize,
    rkyv::Serialize,
    PartialEq,
    serde::Deserialize,
    serde::Serialize,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct IndexedShotMode(pub ShotModeIndex, pub ShotMode);

impl std::ops::Deref for IndexedShotMode {
    type Target = ShotMode;

    fn deref(&self) -> &Self::Target {
        &self.1
    }
}

impl IndexedShotMode {
    pub fn index(&self) -> ShotModeIndex {
        self.0
    }

    pub fn inner(&self) -> &ShotMode {
        &self.1
    }
}

#[derive(
    Debug,
    Clone,
    Archive,
    rkyv::Deserialize,
    rkyv::Serialize,
    PartialEq,
    serde::Deserialize,
    serde::Serialize,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub enum ShotMode {
    Single {
        sounds: Vec<SoundIndex>,
    },
    Burst3 {
        interval: Seconds,
        sounds: Vec<SoundIndex>,
    },
}

#[derive(
    Debug,
    Clone,
    Archive,
    rkyv::Deserialize,
    rkyv::Serialize,
    PartialEq,
    serde::Deserialize,
    serde::Serialize,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub enum ShotModeRaw {
    Single {
        sounds: Vec<String>,
    },
    Burst3 {
        interval: Seconds,
        sounds: Vec<String>,
    },
}

impl ShotModeRaw {
    pub fn resolve_(self, mod_: &Mod) -> Result<ShotMode, super::Error> {
        Ok(match self {
            ShotModeRaw::Single { sounds } => ShotMode::Single {
                sounds: mod_
                    .find_sounds(&sounds)?
                    .iter()
                    .map(|s| s.index())
                    .collect(),
            },
            ShotModeRaw::Burst3 { interval, sounds } => ShotMode::Burst3 {
                interval: interval,
                sounds: mod_
                    .find_sounds(&sounds)?
                    .iter()
                    .map(|s| s.index())
                    .collect(),
            },
        })
    }
}

impl ShotMode {
    pub fn name(&self) -> &str {
        match self {
            ShotMode::Single { sounds: _ } => "Single",
            ShotMode::Burst3 {
                interval: _,
                sounds: _,
            } => "Burst3",
        }
    }

    pub fn rounds(&self) -> usize {
        match self {
            ShotMode::Single { sounds: _ } => 1,
            ShotMode::Burst3 {
                interval: _,
                sounds: _,
            } => 3,
        }
    }

    pub fn interval(&self) -> Seconds {
        match self {
            ShotMode::Single { sounds: _ } => Seconds(0.),
            ShotMode::Burst3 {
                interval,
                sounds: _,
            } => *interval,
        }
    }

    pub fn sounds(&self) -> &Vec<SoundIndex> {
        match self {
            ShotMode::Single { sounds } => sounds,
            ShotMode::Burst3 {
                interval: _,
                sounds,
            } => sounds,
        }
    }
}
