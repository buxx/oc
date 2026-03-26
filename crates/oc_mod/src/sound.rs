use std::path::PathBuf;

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

pub fn load(path: &PathBuf) -> Result<Vec<IndexedSound>, Error> {
    let path = path.join(SOUNDS_FOLDER);
    let mut sounds = vec![];
    let files = std::fs::read_dir(&path);
    let files = files.map_err(|e| Error::CantReadSoundsFolder(path.clone(), e))?;

    for file in files {
        let file = file.map_err(|e| Error::CantReadSoundsFile(e))?;
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
