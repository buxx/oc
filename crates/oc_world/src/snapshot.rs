use std::path::PathBuf;

use derive_more::Constructor;
use oc_individual::Individual;
use oc_projectile::Projectile;
use rkyv::rancor::Error;
use rkyv::{Archive, Deserialize, Serialize};

use crate::tile::Tile;

#[derive(Archive, Deserialize, Serialize, Constructor)]
#[rkyv(compare(PartialEq))]
pub struct Snapshot {
    pub tiles: Vec<Tile>,
    pub individuals: Vec<Individual>,
    pub projectiles: Vec<Projectile>,
}

impl Snapshot {
    pub fn load(path: &PathBuf) -> Result<Self, LoadError> {
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
