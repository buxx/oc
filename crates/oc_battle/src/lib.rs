use std::{fs, io, path::Path};

use bon::Builder;
use getset::Getters;
use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Builder, Deserialize, Getters)]
pub struct Battle {
    #[getset(get = "pub")]
    name: String,
    #[getset(get = "pub")]
    description: String,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Read battle file: {0}")]
    Io(#[from] io::Error),
    #[error("Battle format: {0}")]
    Format(#[from] ron::de::SpannedError),
}

impl Battle {
    pub fn from_ron(path: impl AsRef<Path>) -> Result<Self, Error> {
        let content = fs::read_to_string(path)?;
        Ok(ron::from_str(&content)?)
    }
}
