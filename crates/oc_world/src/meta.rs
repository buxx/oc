use std::path::PathBuf;

use derive_more::Constructor;
use rkyv::{Archive, Deserialize, Serialize};
use thiserror::Error;

#[derive(
    Debug,
    Constructor,
    serde::Deserialize,
    serde::Serialize,
    Clone,
    Archive,
    Deserialize,
    Serialize,
    PartialEq,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct Meta {
    pub id: String,
    // TODO: use it to permit caches invalidation
    pub revision: u32,
}

impl Meta {
    pub fn from_file(path: &PathBuf) -> Result<Self, LoadError> {
        Ok(toml::from_str(&std::fs::read_to_string(path)?)?)
    }

    pub fn folder_name(&self) -> String {
        format!("{}", self.id)
    }
}

#[derive(Debug, Error)]
pub enum LoadError {
    #[error("Io: {0}")]
    Io(#[from] std::io::Error),
    #[error("Toml: {0}")]
    Toml(#[from] toml::de::Error),
}
