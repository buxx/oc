use std::{ops::Deref, path::PathBuf};

use anyhow::Context;
use rkyv::{Archive, Deserialize, Serialize};
use strum_macros::EnumIter;
use thiserror::Error;

pub const PROJECTILES_RON: &str = "projectiles.ron";

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
pub struct ProjectileIndex(pub u32);

impl Deref for ProjectileIndex {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// TODO: use something generic here
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
pub struct IndexedProjectile(pub ProjectileIndex, pub Projectile);

impl Deref for IndexedProjectile {
    type Target = Projectile;

    fn deref(&self) -> &Self::Target {
        &self.1
    }
}

impl IndexedProjectile {
    pub fn id(&self) -> ProjectileIndex {
        self.0
    }

    pub fn inner(&self) -> &Projectile {
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
pub enum Projectile {
    Bullet(Bullet),
}

impl Projectile {
    pub fn label(&self) -> &str {
        match self {
            Projectile::Bullet(bullet) => &bullet.name,
        }
    }

    pub fn is_type(&self, type_: ProjectileType) -> bool {
        match self {
            Projectile::Bullet(_) => matches!(type_, ProjectileType::Bullet),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, EnumIter, Default)]
pub enum ProjectileType {
    #[default]
    Bullet,
}

impl ProjectileType {
    pub fn name(&self) -> &str {
        match self {
            ProjectileType::Bullet => "Bullet",
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
pub struct Bullet {
    name: String,
}

pub fn load(path: &PathBuf) -> Result<Vec<IndexedProjectile>, Error> {
    let path = path.join(PROJECTILES_RON);
    let projectiles = std::fs::read_to_string(&path);
    let projectiles = projectiles.context(format!("Read {}", path.display()))?;
    let projectiles: Vec<Projectile> = ron::from_str(&projectiles)?;

    if projectiles.is_empty() {
        return Err(Error::Empty);
    }

    let projectiles = projectiles
        .into_iter()
        .enumerate()
        .map(|(i, p)| IndexedProjectile(ProjectileIndex(i as u32), p))
        .collect();

    Ok(projectiles)
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Any(#[from] anyhow::Error),
    #[error("Format: {0}")]
    Format(#[from] ron::de::SpannedError),
    #[error("No projectiles defined (require at least one)")]
    Empty,
}
