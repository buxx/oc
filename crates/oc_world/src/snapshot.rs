use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime};

use derive_more::Constructor;
use oc_individual::Individual;
use oc_individual::squad::Squad;
use oc_projectile::Projectile;
use oc_root::WorldConfig;
use oc_root::battle::BattlePhase;
use oc_root::geo::WorldVec2;
use oc_root::side::Side;
use rkyv::rancor::Error;
use rkyv::{
    Archive, Deserialize, Serialize,
    with::{AsUnixTime, Map},
};

use crate::DEFAULT_FIGHT_DURATION;
use crate::spawn::SpawnZoneName;
use crate::support::{PendingSupport, SupportType, Supports};
use crate::tile::Tile;

#[derive(Debug, Archive, Deserialize, Serialize, Constructor)]
pub struct Snapshot {
    pub w: WorldConfig,
    pub tiles: Vec<Tile>,
    pub individuals: Vec<Individual>,
    pub squads: Vec<Squad>,
    pub projectiles: Vec<Projectile>,
    pub phase: BattlePhase,
    pub duration: Duration,
    #[rkyv(with = Map<AsUnixTime>)]
    pub start: Option<SystemTime>,
    pub side_a_spawns: Vec<SpawnZoneName>,
    pub side_b_spawns: Vec<SpawnZoneName>,
    pub supports: Supports,
    pub pending_supports: Vec<SnapshotPendingSupport>,
}

/// Persistable form of PendingSupport (Instant can't be persisted)
#[derive(Debug, Clone, Archive, Deserialize, Serialize)]
pub struct SnapshotPendingSupport {
    pub type_: SupportType,
    pub side: Side,
    pub position: WorldVec2,
    #[rkyv(with = Map<AsUnixTime>)]
    pub start: Option<SystemTime>,
}

impl From<&PendingSupport> for SnapshotPendingSupport {
    fn from(value: &PendingSupport) -> Self {
        Self {
            type_: value.type_,
            side: value.side,
            position: value.position,
            start: value.start.map(|start| {
                SystemTime::now()
                    .checked_sub(start.elapsed())
                    .unwrap_or_else(SystemTime::now)
            }),
        }
    }
}

impl From<SnapshotPendingSupport> for PendingSupport {
    fn from(value: SnapshotPendingSupport) -> Self {
        let start = value.start.map(|start| {
            let elapsed = start.elapsed().unwrap_or_default();
            Instant::now()
                .checked_sub(elapsed)
                .unwrap_or_else(Instant::now)
        });
        Self {
            type_: value.type_,
            side: value.side,
            position: value.position,
            start,
        }
    }
}

impl Snapshot {
    pub fn empty(w: WorldConfig) -> Self {
        Self {
            w,
            tiles: vec![],
            individuals: vec![],
            squads: vec![],
            projectiles: vec![],
            phase: BattlePhase::Deployment,
            duration: DEFAULT_FIGHT_DURATION,
            start: None,
            side_a_spawns: vec![],
            side_b_spawns: vec![],
            supports: Supports::default(),
            pending_supports: vec![],
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

    pub fn with_phase(mut self, value: BattlePhase) -> Self {
        self.phase = value;
        self
    }

    pub fn with_supports(mut self, value: Supports) -> Self {
        self.supports = value;
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
