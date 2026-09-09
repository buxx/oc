use std::path::PathBuf;

use derive_more::Constructor;
use oc_individual::Individual;
use oc_individual::squad::Squad;
use oc_projectile::Projectile;
use oc_root::WorldConfig;
use rkyv::rancor::Error;
use rkyv::{Archive, Deserialize, Serialize};

use crate::tile::Tile;

#[derive(Debug, Archive, Deserialize, Serialize, Constructor)]
#[rkyv(compare(PartialEq))]
pub struct Snapshot {
    pub w: WorldConfig,
    pub tiles: Vec<Tile>,
    pub individuals: Vec<Individual>,
    pub squads: Vec<Squad>,
    pub projectiles: Vec<Projectile>,
}

impl Snapshot {
    pub fn empty(w: WorldConfig) -> Self {
        Self {
            w,
            tiles: vec![],
            individuals: vec![],
            squads: vec![],
            projectiles: vec![],
        }
    }

    pub fn load(path: &PathBuf) -> Result<Self, LoadError> {
        tracing::info!("Load snapshot from {}", path.display());
        let bytes_ = std::fs::read(path);
        let bytes_ = bytes_.map_err(|e| LoadError::SourceIo(path.clone(), e))?;
        let mut bytes: rkyv::util::AlignedVec = rkyv::util::AlignedVec::with_capacity(bytes_.len());
        bytes.extend_from_slice(&bytes_);
        let snapshot = rkyv::access::<ArchivedSnapshot, Error>(&bytes[..]);
        let snapshot = snapshot.map_err(|e| LoadError::Format(path.clone(), e))?;
        let snapshot = rkyv::deserialize::<Snapshot, Error>(snapshot);
        let snapshot = snapshot.map_err(|e| LoadError::Format(path.clone(), e))?;
        Ok(snapshot)
    }

    pub fn with_tiles(mut self, tiles: Vec<Tile>) -> Self {
        self.tiles = tiles;
        self
    }

    pub fn with_individuals(mut self, individuals: Vec<Individual>) -> Self {
        self.individuals = individuals;
        self
    }

    pub fn with_squads(mut self, squads: Vec<Squad>) -> Self {
        self.squads = squads;
        self
    }

    pub fn with_projectiles(mut self, projectiles: Vec<Projectile>) -> Self {
        self.projectiles = projectiles;
        self
    }

    pub fn save(&self, to: &PathBuf) -> Result<(), SaveError> {
        let bytes = rkyv::to_bytes::<Error>(self);
        let bytes = bytes.map_err(|e| SaveError::Encode(to.clone(), e))?;
        let write = std::fs::write(to, &bytes);
        write.map_err(|e| SaveError::Write(to.clone(), e))?;
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    #[error("Load snapshot file '{0}' io error: {1}")]
    SourceIo(PathBuf, std::io::Error),
    #[error("Load snapshot file '{0}' format error: {1}")]
    Format(PathBuf, rkyv::rancor::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum SaveError {
    #[error("Save snapshot to file '{0}' encode error: {1}")]
    Encode(PathBuf, rkyv::rancor::Error),
    #[error("Save snapshot to file '{0}' write error: {1}")]
    Write(PathBuf, std::io::Error),
}
