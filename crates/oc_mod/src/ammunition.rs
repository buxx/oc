use std::{ops::Deref, path::PathBuf};

use anyhow::Context;
use rkyv::Archive;
use strum_macros::EnumIter;
use thiserror::Error;

pub const PROJECTILES_RON: &str = "amunitions.ron";

#[derive(
    Debug,
    Clone,
    Copy,
    Archive,
    rkyv::Deserialize,
    rkyv::Serialize,
    PartialEq,
    Hash,
    Eq,
    serde::Deserialize,
    serde::Serialize,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct AmmunitionIndex(pub u32);

impl Deref for AmmunitionIndex {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(
    Debug, Clone, Archive, rkyv::Deserialize, rkyv::Serialize, serde::Deserialize, serde::Serialize,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct IndexedAmmunition(pub AmmunitionIndex, pub Ammunition);

impl Deref for IndexedAmmunition {
    type Target = Ammunition;

    fn deref(&self) -> &Self::Target {
        &self.1
    }
}

impl PartialEq for IndexedAmmunition {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for IndexedAmmunition {}

impl std::hash::Hash for IndexedAmmunition {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl IndexedAmmunition {
    pub fn index(&self) -> AmmunitionIndex {
        self.0
    }

    pub fn inner(&self) -> &Ammunition {
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
pub enum Ammunition {
    Cartridge(Cartridge),
}

impl Ammunition {
    pub fn name(&self) -> &str {
        match self {
            Ammunition::Cartridge(bullet) => &bullet.name,
        }
    }

    pub fn is_type(&self, type_: AmmunitionType) -> bool {
        match self {
            Ammunition::Cartridge(_) => matches!(type_, AmmunitionType::Cartridge),
        }
    }

    pub fn model(&self) -> &String {
        match self {
            Ammunition::Cartridge(cartridge) => &cartridge.model,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, EnumIter, Default)]
pub enum AmmunitionType {
    #[default]
    Cartridge,
}

impl AmmunitionType {
    pub fn name(&self) -> &str {
        match self {
            AmmunitionType::Cartridge => "Cartridge",
        }
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
pub struct Cartridge {
    name: String,
    model: String,
}

// TODO: use something generic here (bullet/weapon/etc)
pub fn load(path: &PathBuf) -> Result<Vec<IndexedAmmunition>, Error> {
    let path = path.join(PROJECTILES_RON);
    let amunitions = std::fs::read_to_string(&path);
    let amunitions = amunitions.context(format!("Read {}", path.display()))?;
    let amunitions: Vec<Ammunition> = ron::from_str(&amunitions)?;

    if amunitions.is_empty() {
        return Err(Error::Empty);
    }

    let amunitions = amunitions
        .into_iter()
        .enumerate()
        .map(|(i, p)| IndexedAmmunition(AmmunitionIndex(i as u32), p))
        .collect();

    Ok(amunitions)
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Any(#[from] anyhow::Error),
    #[error("Format: {0}")]
    Format(#[from] ron::de::SpannedError),
    #[error("No amunitions defined (require at least one)")]
    Empty,
}
