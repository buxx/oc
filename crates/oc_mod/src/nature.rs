use std::{ops::Deref, path::PathBuf};

use anyhow::Context;
use oc_root::{WorldConfig, opacity::Opacity, physics::Meters};
use rkyv::Archive;
use thiserror::Error;

pub const NATURES_RON: &str = "natures.ron";

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
    Eq,
    Hash,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct NatureIndex(pub u16);

impl Deref for NatureIndex {
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
pub struct IndexedNature(pub NatureIndex, pub Nature);

impl Deref for IndexedNature {
    type Target = Nature;

    fn deref(&self) -> &Self::Target {
        &self.1
    }
}

impl IndexedNature {
    pub fn index(&self) -> NatureIndex {
        self.0
    }

    pub fn inner(&self) -> &Nature {
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
pub struct Nature {
    pub name: String,
    pub opacity: f32,
    pub z: Meters,
}

impl Nature {
    pub fn opacity(&self, w: &WorldConfig, z: f32) -> Opacity {
        // Negative means "in the ground"
        if z < 0.0 {
            return Opacity(1.0);
        }

        // Above means in the air, so no opacity
        // TODO: this should be computed once (do it al mod load ? need WorldConfig ...)
        let height = self.z.0 * w.geo_pixels_per_meters;
        if z > height {
            return Opacity(0.0);
        }

        Opacity(self.opacity)
    }
}

// TODO: use something generic here (bullet/weapon/etc)
pub fn load(path: &PathBuf) -> Result<Vec<IndexedNature>, Error> {
    let path = path.join(NATURES_RON);
    let natures = std::fs::read_to_string(&path);
    let natures = natures.context(format!("Read {}", path.display()))?;
    let natures: Vec<Nature> = ron::from_str(&natures)?;

    if natures.is_empty() {
        return Err(Error::Empty);
    }

    let natures = natures
        .into_iter()
        .enumerate()
        .map(|(i, p)| IndexedNature(NatureIndex(i as u16), p))
        .collect();

    Ok(natures)
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Any(#[from] anyhow::Error),
    #[error("Format: {0}")]
    InvalidFormat(String),
    #[error("Format: {0}")]
    Format(#[from] ron::de::SpannedError),
    #[error("No natures defined (require at least one)")]
    Empty,
}
