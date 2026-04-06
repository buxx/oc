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
    #[serde(skip, default)]
    pub name: String,
    pub revision: u32,
}

impl Meta {
    pub fn from_file(path: &PathBuf) -> Result<Self, LoadError> {
        // TODO
        let name = path
            .parent()
            .unwrap()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        let mut meta: Self = toml::from_str(&std::fs::read_to_string(path)?)?;
        meta.name = name;
        Ok(meta)
    }

    pub fn canonical(&self) -> String {
        format!("{}_{}", self.name, self.revision)
    }

    pub fn archive(&self) -> String {
        format!("{}.tar.gz", self.canonical())
    }
}

#[derive(Debug, Error)]
pub enum LoadError {
    #[error("Io: {0}")]
    Io(#[from] std::io::Error),
    #[error("Toml: {0}")]
    Toml(#[from] toml::de::Error),
}
