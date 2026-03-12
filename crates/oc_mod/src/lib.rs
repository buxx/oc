use std::path::PathBuf;

use anyhow::Context;
use flate2::{Compression, write::GzEncoder};
use rkyv::{Archive, Deserialize, Serialize};
use ron;
use tar::Builder;
use thiserror::Error;

use crate::projectiles::ProjectileIndex;

pub mod projectiles;

pub const MOD_DIR: &str = "mods";
pub const MOD_RON: &str = "mod.ron";

#[derive(
    Debug, Clone, Archive, Deserialize, Serialize, PartialEq, serde::Deserialize, serde::Serialize,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct Mod {
    name: String,
    version: u32,
    #[serde(skip, default)]
    pub projectiles: Vec<projectiles::IndexedProjectile>,
}

impl Mod {
    pub fn load(path: &PathBuf, cache_: Option<&PathBuf>) -> Result<Self, Error> {
        let mut mod_ = load_mod(&path)?;

        mod_.projectiles = projectiles::load(&path)?;

        if let Some(cache_) = cache_ {
            cache(&mod_, &path, cache_)?;
        }

        Ok(mod_)
    }

    pub fn canonical(&self) -> String {
        format!("{}_{}", self.name, self.version)
    }

    pub fn archive(&self) -> String {
        format!("{}.tar.gz", self.canonical())
    }
}

fn load_mod(path: &PathBuf) -> Result<Mod, ModError> {
    let path = path.join(MOD_RON);
    let mod_ = std::fs::read_to_string(&path);
    let mod_ = mod_.context(format!("Read {}", path.display()))?;
    Ok(ron::from_str(&mod_)?)
}

fn cache(mod_: &Mod, path: &PathBuf, cache: &PathBuf) -> Result<(), CacheError> {
    let cache = cache.join(MOD_DIR);
    std::fs::create_dir_all(&cache).context(format!("Create dirs {}", cache.display()))?;
    let cache = cache.join(mod_.archive());
    if !std::fs::exists(&cache).context(format!("Test if {} exists", cache.display()))? {
        tracing::info!("Caching {} to {}", &mod_.name, cache.display());

        let file = std::fs::File::create(&cache);
        let file = file.context(format!("Create file {}", cache.display()))?;
        let encoder = GzEncoder::new(file, Compression::default());
        let mut builder = Builder::new(encoder);

        builder.append_dir_all(&mod_.name, &path).context(format!(
            "Create archive from under '{}' from '{}'",
            &mod_.name,
            &path.display()
        ))?;
        builder
            .finish()
            .context(format!("Finish builder ({})", &path.display()))?;
    }

    Ok(())
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Caching: {0}")]
    Cache(#[from] CacheError),
    #[error("mod.ron: {0}")]
    Mod(#[from] ModError),
    #[error("Projectiles: {0}")]
    Projectiles(#[from] projectiles::Error),
}

#[derive(Debug, Error)]
pub enum ModError {
    #[error("{0}")]
    Any(#[from] anyhow::Error),
    #[error("Format: {0}")]
    Format(#[from] ron::de::SpannedError),
}

#[derive(Debug, Error)]
pub enum CacheError {
    #[error("{0}")]
    Any(#[from] anyhow::Error),
}
