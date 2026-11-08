use std::{ops::Deref, path::Path};

use crate::{Mod, sound::SoundIndex};
use anyhow::Context;
use oc_root::{explosion::Explosion, physics::Meters};
use rkyv::Archive;
use thiserror::Error;

pub const GRENADES_RON: &str = "grenades.ron";

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
pub struct GrenadeIndex(pub u16);

impl Deref for GrenadeIndex {
    type Target = u16;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    Archive,
    rkyv::Deserialize,
    rkyv::Serialize,
    PartialEq,
    Eq,
    serde::Deserialize,
    serde::Serialize,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub enum GrenadeNature {
    Frag,
    Smoke,
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
pub struct IndexedGrenade(pub GrenadeIndex, pub Grenade);

impl Deref for IndexedGrenade {
    type Target = Grenade;

    fn deref(&self) -> &Self::Target {
        &self.1
    }
}

impl IndexedGrenade {
    pub fn index(&self) -> GrenadeIndex {
        self.0
    }

    pub fn inner(&self) -> &Grenade {
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
pub struct Grenade {
    pub name: String,
    pub blast_rayon: Meters,
    pub shrapnel_rayon: Meters,
    pub shrapnel_quantity: u32,
    pub nature: GrenadeNature,
    pub sounds: Vec<SoundIndex>,
    /// Smoke only: seconds the smoke keeps its full opacity
    pub smoke_duration: f32,
    /// Smoke only: seconds, after `smoke_duration`, the smoke takes to disappear
    pub smoke_fade_duration: f32,
}

#[derive(Debug, serde::Deserialize)]
struct GrenadeRaw {
    name: String,
    blast_rayon: Meters,
    shrapnel_rayon: Meters,
    shrapnel_quantity: u32,
    nature: GrenadeNature,
    #[serde(default)]
    sounds: Vec<String>,
    #[serde(default)]
    smoke_duration: f32,
    #[serde(default)]
    smoke_fade_duration: f32,
}

impl Grenade {
    pub fn pick_sound(&self) -> Option<SoundIndex> {
        crate::sound::pick(&self.sounds)
    }
}

impl From<Grenade> for Explosion {
    fn from(value: Grenade) -> Self {
        match value.nature {
            GrenadeNature::Frag => Self {
                kill_rayon: value.blast_rayon,
                stunned_rayon: value.blast_rayon,
                shrapnel_rayon: value.shrapnel_rayon,
                destruction_rayon: Meters(0.0),
                destruction_force: 0.0,
                shrapnel_quantity: value.shrapnel_quantity,
                penetration_force: 0.0,
            },
            GrenadeNature::Smoke => Self {
                kill_rayon: Meters(0.0),
                stunned_rayon: Meters(0.0),
                shrapnel_rayon: Meters(0.0),
                destruction_rayon: Meters(0.0),
                destruction_force: 0.0,
                shrapnel_quantity: 0,
                penetration_force: 0.0,
            },
        }
    }
}

pub fn load(path: &Path, mod_: &Mod) -> Result<Vec<IndexedGrenade>, Error> {
    let path = path.join(GRENADES_RON);
    let grenades = std::fs::read_to_string(&path);
    let grenades = grenades.context(format!("Read {}", path.display()))?;
    let grenades: Vec<GrenadeRaw> = ron::from_str(&grenades)?;

    grenades
        .into_iter()
        .enumerate()
        .map(|(i, g)| {
            let sounds = g
                .sounds
                .iter()
                .map(|name| mod_.find_sound(name))
                .collect::<Result<Vec<_>, _>>()
                .map_err(Box::new)?;
            let grenade = Grenade {
                name: g.name,
                blast_rayon: g.blast_rayon,
                shrapnel_rayon: g.shrapnel_rayon,
                shrapnel_quantity: g.shrapnel_quantity,
                nature: g.nature,
                sounds,
                smoke_duration: g.smoke_duration,
                smoke_fade_duration: g.smoke_fade_duration,
            };
            Ok(IndexedGrenade(GrenadeIndex(i as u16), grenade))
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
