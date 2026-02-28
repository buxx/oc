use std::path::PathBuf;

use derive_more::Constructor;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Constructor, Deserialize, Serialize)]
pub struct Meta {
    pub id: Uuid,
    pub revision: usize,
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
