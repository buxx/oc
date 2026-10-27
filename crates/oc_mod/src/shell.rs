use std::{ops::Deref, path::Path};

use anyhow::Context;
use oc_root::{explosion::Explosion, physics::Meters};

use crate::{Mod, sound::SoundIndex};
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
    pub sounds: Vec<SoundIndex>,
}

#[derive(Debug, serde::Deserialize)]
struct ShellRaw {
    name: String,
    kill_rayon: Meters,
    stunned_rayon: Meters,
    shrapnel_rayon: Meters,
    shrapnel_quantity: u32,
    penetration_force: f32,
    #[serde(default)]
    sounds: Vec<String>,
}

impl Shell {
    pub fn pick_sound(&self) -> Option<SoundIndex> {
        crate::sound::pick(&self.sounds)
    }
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

pub fn load(path: &Path, mod_: &Mod) -> Result<Vec<IndexedShell>, Error> {
    let path = path.join(SHELLS_RON);
    let shells = std::fs::read_to_string(&path);
    let shells = shells.context(format!("Read {}", path.display()))?;
    let shells: Vec<ShellRaw> = ron::from_str(&shells)?;

    shells
        .into_iter()
        .enumerate()
        .map(|(i, s)| {
            let sounds = s
                .sounds
                .iter()
                .map(|name| mod_.find_sound(name))
                .collect::<Result<Vec<_>, _>>()
                .map_err(Box::new)?;
            let shell = Shell {
                name: s.name,
                kill_rayon: s.kill_rayon,
                stunned_rayon: s.stunned_rayon,
                shrapnel_rayon: s.shrapnel_rayon,
                shrapnel_quantity: s.shrapnel_quantity,
                penetration_force: s.penetration_force,
                sounds,
            };
            Ok(IndexedShell(ShellIndex(i as u16), shell))
        })
        .collect()
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Any(#[from] anyhow::Error),
    #[error("Format: {0}")]
    Format(#[from] ron::de::SpannedError),
    #[error("{0}")]
    Mod(#[from] Box<crate::Error>),
}
