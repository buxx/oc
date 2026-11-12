use std::{
    fs, io,
    path::{Path, PathBuf},
};

use bon::Builder;
use serde::Deserialize;
use thiserror::Error;

pub mod check;
pub mod scenario;

pub use scenario::Scenario;

#[derive(Debug, Builder)]
pub struct Battle {
    name: String,
    description: String,
    /// Map folder name
    map: String,
    #[builder(default)]
    scenarios: Vec<Scenario>,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Read file {path}: {source}")]
    Io { path: PathBuf, source: io::Error },
    #[error("Format of {path}: {source}")]
    Format {
        path: PathBuf,
        source: ron::de::SpannedError,
    },
}

#[derive(Deserialize)]
struct BattleRaw {
    name: String,
    description: String,
    map: String,
    scenarios: Vec<String>,
}

pub fn read_ron<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, Error> {
    let content = fs::read_to_string(path).map_err(|source| Error::Io {
        path: path.to_path_buf(),
        source,
    })?;
    ron::from_str(&content).map_err(|source| Error::Format {
        path: path.to_path_buf(),
        source,
    })
}

impl Battle {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn map(&self) -> &str {
        &self.map
    }

    pub fn scenarios(&self) -> &[Scenario] {
        &self.scenarios
    }

    pub fn from_ron_file(path: impl AsRef<Path>) -> Result<Self, Error> {
        let path = path.as_ref();
        let dir = path.parent().unwrap_or(Path::new("."));
        let raw: BattleRaw = read_ron(path)?;

        let scenarios = raw
            .scenarios
            .iter()
            .map(|file| Scenario::from_ron_file(dir.join(file)))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            name: raw.name,
            description: raw.description,
            map: raw.map,
            scenarios,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_ron_file_demo2() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../battles/demo2/demo2.ron");
        let battle = Battle::from_ron_file(path).unwrap();
        assert_eq!(battle.name(), "Demo2");
        assert_eq!(battle.scenarios().len(), 1);
        let scenario = &battle.scenarios()[0];
        assert_eq!(scenario.deploy_a().squads.len(), 2);
        assert_eq!(scenario.deploy_b().squads.len(), 2);
    }
}
