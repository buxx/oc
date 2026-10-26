use std::{path::Path, time::Duration};

use anyhow::Context;
use oc_root::physics::Meters;
use rkyv::Archive;
use thiserror::Error;

pub const SUPPORT_RON: &str = "support.ron";

use crate::shell::ShellIndex;

/// Support as written in support.ron (shells referenced by name)
#[derive(
    Debug,
    Clone,
    Default,
    Archive,
    rkyv::Deserialize,
    rkyv::Serialize,
    PartialEq,
    serde::Deserialize,
    serde::Serialize,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct SupportRaw {
    pub naval_bombard_shell: String,
    pub naval_bombard_series: u32,
    pub naval_bombard_shells_by_serie: u32,
    pub naval_bombard_radius: Meters,
    pub naval_bombard_prepare_seconds: u64,
    pub naval_bombard_serie_interval_ms: u64,
    pub naval_bombard_shell_interval_ms: u64,
}

#[derive(
    Debug,
    Clone,
    Default,
    Archive,
    rkyv::Deserialize,
    rkyv::Serialize,
    PartialEq,
    serde::Deserialize,
    serde::Serialize,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct Support {
    pub naval_bombard_shell: ShellIndex,
    pub naval_bombard_series: u32,
    pub naval_bombard_shells_by_serie: u32,
    pub naval_bombard_radius: Meters,
    pub naval_bombard_prepare_seconds: u64,
    pub naval_bombard_serie_interval_ms: u64,
    pub naval_bombard_shell_interval_ms: u64,
}

impl Support {
    pub fn naval_bombard_prepare(&self) -> Duration {
        Duration::from_secs(self.naval_bombard_prepare_seconds)
    }

    pub fn naval_bombard_serie_interval(&self) -> Duration {
        Duration::from_millis(self.naval_bombard_serie_interval_ms)
    }

    pub fn naval_bombard_shell_interval(&self) -> Duration {
        Duration::from_millis(self.naval_bombard_shell_interval_ms)
    }
}

pub fn load(path: &Path) -> Result<SupportRaw, Error> {
    let path = path.join(SUPPORT_RON);
    let support = std::fs::read_to_string(&path);
    let support = support.context(format!("Read {}", path.display()))?;

    Ok(ron::from_str(&support)?)
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Any(#[from] anyhow::Error),
    #[error("Format: {0}")]
    Format(#[from] ron::de::SpannedError),
}
