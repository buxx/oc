use std::{ops::Deref, path::Path};

use anyhow::Context;
use oc_root::{explosion::Explosion, physics::Meters};
use rkyv::Archive;
use thiserror::Error;

pub const SHELLS_RON: &str = "shells.ron";

#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    Archive,
    rkyv::Deserialize,
    rkyv::Serialize,
    PartialEq,
    serde::Deserialize,
    serde::Serialize,
    Eq,
    Hash,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct ShellIndex(pub u16);

impl Deref for ShellIndex {
    type Target = u16;

    fn deref(&self) -> &Self::Target {
        &self.0
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
pub struct IndexedShell(pub ShellIndex, pub Shell);

impl Deref for IndexedShell {
    type Target = Shell;

    fn deref(&self) -> &Self::Target {
        &self.1
    }
}

impl IndexedShell {
    pub fn index(&self) -> ShellIndex {
        self.0
    }

    pub fn inner(&self) -> &Shell {
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
pub struct Shell {
    pub name: String,
    pub kill_rayon: Meters,
    pub stunned_rayon: Meters,
    pub shrapnel_rayon: Meters,
    pub shrapnel_quantity: u32,
    pub penetration_force: f32,
}

impl From<Shell> for Explosion {
    fn from(value: Shell) -> Self {
        Self {
            kill_rayon: value.kill_rayon,
            stunned_rayon: value.stunned_rayon,
            shrapnel_rayon: value.shrapnel_rayon,
            shrapnel_quantity: value.shrapnel_quantity,
            penetration_force: value.penetration_force,
        }
    }
}

pub fn load(path: &Path) -> Result<Vec<IndexedShell>, Error> {
    let path = path.join(SHELLS_RON);
    let shells = std::fs::read_to_string(&path);
    let shells = shells.context(format!("Read {}", path.display()))?;
    let shells: Vec<Shell> = ron::from_str(&shells)?;

    Ok(shells
        .into_iter()
        .enumerate()
        .map(|(i, s)| IndexedShell(ShellIndex(i as u16), s))
        .collect())
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Any(#[from] anyhow::Error),
    #[error("Format: {0}")]
    Format(#[from] ron::de::SpannedError),
}
