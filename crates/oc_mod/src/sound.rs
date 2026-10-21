use std::path::{Path, PathBuf};

use derive_more::{Constructor, Deref};
use rkyv::Archive;
use thiserror::Error;

pub const SOUNDS_FOLDER: &str = "sounds";

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
    Deref,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct SoundIndex(pub u32);

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
pub struct IndexedSound(pub SoundIndex, pub Sound);

impl std::ops::Deref for IndexedSound {
    type Target = Sound;

    fn deref(&self) -> &Self::Target {
        &self.1
    }
}

impl IndexedSound {
    pub fn index(&self) -> SoundIndex {
        self.0
    }

    pub fn inner(&self) -> &Sound {
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
    Constructor,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct Sound {
    pub name: String,
}

/// Sounds groups as written in mod.ron (sounds file names)
#[derive(
    Debug,
    Clone,
    Default,
    Archive,
    rkyv::Deserialize,
    rkyv::Serialize,
    PartialEq,
    serde::Deserialize,
    serde::Serialize,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
#[serde(default)]
pub struct SoundGroupsNames {
    pub die: Vec<String>,
    pub main_click: Option<String>,
    pub secondary_click: Option<String>,
    pub deployment_start: Option<String>,
    pub fight_start: Option<String>,
    pub bullet_impact_ground: Vec<String>,
    pub bullet_wall_impact: Vec<String>,
    pub bullet_metal_impact: Vec<String>,
}

/// Sounds groups resolved to sounds indexes
#[derive(Debug, Clone, Default, Archive, rkyv::Deserialize, rkyv::Serialize, PartialEq)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct SoundGroups {
    pub die: Vec<SoundIndex>,
    pub main_click: Option<SoundIndex>,
    pub secondary_click: Option<SoundIndex>,
    pub deployment_start: Option<SoundIndex>,
    pub fight_start: Option<SoundIndex>,
    pub bullet_impact_ground: Vec<SoundIndex>,
    pub bullet_wall_impact: Vec<SoundIndex>,
    pub bullet_metal_impact: Vec<SoundIndex>,
}

/// Pick randomly one of the given sounds
pub fn pick(sounds: &[SoundIndex]) -> Option<SoundIndex> {
    if sounds.is_empty() {
        return None;
    }
    sounds.get(fastrand::usize(..sounds.len())).copied()
}

pub fn load(path: &Path) -> Result<Vec<IndexedSound>, Error> {
    let path = path.join(SOUNDS_FOLDER);
    let mut sounds = vec![];
    let files = std::fs::read_dir(&path);
    let files = files.map_err(|e| Error::CantReadSoundsFolder(path.clone(), e))?;

    for file in files {
        let file = file.map_err(Error::CantReadSoundsFile)?;
        let file = file.path();
        let name = file.file_name();
        let name = name.ok_or(Error::AbnormalSoundFileName(file.clone()))?;
        let name = name.to_str();
        let name = name.ok_or(Error::AbnormalSoundFileName(file.clone()))?;
        sounds.push(Sound::new(name.to_string()));
    }

    let sounds = sounds
        .into_iter()
        .enumerate()
        .map(|(i, p)| IndexedSound(SoundIndex(i as u32), p))
        .collect();

    Ok(sounds)
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Can't read sounds folder ({0}): {1}")]
    CantReadSoundsFolder(PathBuf, std::io::Error),
    #[error("Can't read sounds file: {0}")]
    CantReadSoundsFile(std::io::Error),
    #[error("Abnormal sound file: {0}")]
    AbnormalSoundFileName(PathBuf),
}
