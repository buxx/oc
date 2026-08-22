use std::{ops::Deref, path::PathBuf};

use anyhow::Context;
use rkyv::Archive;
use strum_macros::EnumIter;
use thiserror::Error;

pub const MAGAZINES_RON: &str = "magazine.ron";

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
)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct MagazineIndex(pub u32);

impl Deref for MagazineIndex {
    type Target = u32;

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
pub struct IndexedMagazine(pub MagazineIndex, pub Magazine);

impl Deref for IndexedMagazine {
    type Target = Magazine;

    fn deref(&self) -> &Self::Target {
        &self.1
    }
}

impl IndexedMagazine {
    pub fn index(&self) -> MagazineIndex {
        self.0
    }

    pub fn inner(&self) -> &Magazine {
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
pub enum Magazine {
    Cartridge(Cartridge),
}

impl Magazine {
    pub fn name(&self) -> &str {
        match self {
            Magazine::Cartridge(cartridge) => &cartridge.name,
        }
    }

    pub fn is_type(&self, type_: MagazineType) -> bool {
        match self {
            Magazine::Cartridge(_) => matches!(type_, MagazineType::Cartridge),
        }
    }

    pub fn capacity(&self) -> u16 {
        match self {
            Magazine::Cartridge(cartridge) => cartridge.capacity,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, EnumIter, Default)]
pub enum MagazineType {
    #[default]
    Cartridge,
}

impl MagazineType {
    pub fn name(&self) -> &str {
        match self {
            MagazineType::Cartridge => "Cartridge",
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
    capacity: u16,
    accept: Vec<String>,
}

// TODO: use something generic here (bullet/weapon/etc)
pub fn load(path: &PathBuf) -> Result<Vec<IndexedMagazine>, Error> {
    let path = path.join(MAGAZINES_RON);
    let amunitions = std::fs::read_to_string(&path);
    let amunitions = amunitions.context(format!("Read {}", path.display()))?;
    let amunitions: Vec<Magazine> = ron::from_str(&amunitions)?;

    if amunitions.is_empty() {
        return Err(Error::Empty);
    }

    let amunitions = amunitions
        .into_iter()
        .enumerate()
        .map(|(i, p)| IndexedMagazine(MagazineIndex(i as u32), p))
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
