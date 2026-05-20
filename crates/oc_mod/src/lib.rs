use std::path::PathBuf;

use anyhow::Context;
use derive_more::Constructor;
use flate2::{Compression, write::GzEncoder};
use oc_root::{files, physics::Meters};
use rkyv::{Archive, Deserialize, Serialize};
use tar::Builder;
use thiserror::Error;

use crate::{
    ammunition::{Ammunition, AmmunitionIndex, IndexedAmmunition},
    nature::{Nature, NatureIndex},
    sound::IndexedSound,
    weapons::{Weapon, WeaponIndex},
};

pub mod ammunition;
pub mod armament;
pub mod nature;
pub mod sound;
pub mod weapons;

pub const MOD_RON: &str = "mod.ron";

pub const DEFAULT_HUMAN_DEFAULT_STAND_UP_FIRE_METERS: Meters = Meters(1.5);

#[derive(
    Debug,
    Clone,
    Archive,
    Deserialize,
    Serialize,
    PartialEq,
    serde::Deserialize,
    serde::Serialize,
    Constructor,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct Mod {
    name: String,
    version: u32,
    #[serde(skip, default)]
    pub natures: Vec<nature::IndexedNature>,
    #[serde(skip, default)]
    pub sounds: Vec<sound::IndexedSound>,
    #[serde(skip, default)]
    pub ammunitions: Vec<ammunition::IndexedAmmunition>,
    #[serde(skip, default)]
    pub weapons: Vec<weapons::IndexedWeapon>,
    // Below game specs
    #[serde(default = "default_human_default_stand_up_fire_meters")]
    pub human_default_stand_up_fire_meters: f32,
}

impl Mod {
    pub fn load(path: &PathBuf, cache_: Option<&PathBuf>) -> Result<Self, Error> {
        let mut mod_ = load_mod(path)?;

        mod_.natures = nature::load(path)?;
        mod_.sounds = sound::load(path)?;
        mod_.ammunitions = ammunition::load(path)?;
        mod_.weapons = weapons::load(path, &mod_)?;

        // TODO: centralize caching at server startup
        if let Some(cache_) = cache_ {
            cache(&mod_, path, cache_)?;
        }

        Ok(mod_)
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn canonical(&self) -> String {
        format!("{}_{}", self.name, self.version)
    }

    pub fn archive(&self) -> String {
        format!("{}.tar.gz", self.canonical())
    }

    fn amunitions_from_names(
        &self,
        amunitions: Vec<String>,
    ) -> Result<Vec<&IndexedAmmunition>, Error> {
        amunitions
            .iter()
            .map(|amunition| {
                self.ammunitions
                    .iter()
                    .find(|a| a.name() == amunition)
                    .ok_or(Error::UnknownAmunitionName(amunition.clone()))
            })
            .collect::<Result<Vec<&IndexedAmmunition>, Error>>()
    }

    pub fn ammunition(&self, index: AmmunitionIndex) -> &Ammunition {
        &self.ammunitions[index.0 as usize]
    }

    pub fn weapon(&self, index: WeaponIndex) -> &Weapon {
        &self.weapons[index.0 as usize]
    }

    fn find_sounds(&self, sounds: &[String]) -> Result<Vec<&IndexedSound>, Error> {
        sounds
            .iter()
            .map(|name| {
                self.sounds
                    .iter()
                    .find(|s| &s.name == name)
                    .ok_or(Error::UnknownSoundName(name.clone()))
            })
            .collect()
    }

    pub fn sound(&self, sound: sound::SoundIndex) -> &IndexedSound {
        &self.sounds[sound.0 as usize]
    }

    pub fn nature(&self, index: NatureIndex) -> &Nature {
        &self.natures[index.0 as usize]
    }

    pub fn nature_index(&self, name: &str) -> Option<NatureIndex> {
        self.natures
            .iter()
            .find(|nature| nature.name == name)
            .map(|nature| nature.index())
    }
}

fn load_mod(path: &PathBuf) -> Result<Mod, ModError> {
    let path = path.join(MOD_RON);
    let mod_ = std::fs::read_to_string(&path);
    let mod_ = mod_.context(format!("Read {}", path.display()))?;
    Ok(ron::from_str(&mod_)?)
}

// TODO: centralize caching at server startup
fn cache(mod_: &Mod, path: &PathBuf, cache: &PathBuf) -> Result<(), CacheError> {
    let files = files::Files::new(mod_.canonical(), "".to_string()).into_server(cache.clone());
    let cache = files.mod_archive();
    if !std::fs::exists(&cache).context(format!("Test if {} exists", cache.display()))? {
        tracing::info!("Caching {} to {}", &mod_.name, cache.display());

        let file = std::fs::File::create(&cache);
        let file = file.context(format!("Create file {}", cache.display()))?;
        let encoder = GzEncoder::new(file, Compression::default());
        let mut builder = Builder::new(encoder);

        builder.append_dir_all(&mod_.name, path).context(format!(
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

pub trait PickSound<S> {
    fn pick_sound(&self, specs: S) -> Option<sound::SoundIndex>;
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Caching: {0}")]
    Cache(#[from] CacheError),
    #[error("mod.ron: {0}")]
    Mod(#[from] ModError),
    #[error("Amunitions: {0}")]
    Amunitions(#[from] ammunition::Error),
    #[error("Unknown amunitions name: {0}")]
    UnknownAmunitionName(String),
    #[error("Unknown sound name: {0}")]
    UnknownSoundName(String),
    #[error("Weapons: {0}")]
    Natures(#[from] nature::Error),
    #[error("Natures: {0}")]
    Weapons(#[from] weapons::Error),
    #[error("Sounds: {0}")]
    Sounds(#[from] sound::Error),
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

fn default_human_default_stand_up_fire_meters() -> f32 {
    DEFAULT_HUMAN_DEFAULT_STAND_UP_FIRE_METERS.0
}
