use std::path::Path;

use bon::Builder;
use oc_deployment::generator::profile::Profile;
use oc_world::spawn::SpawnZoneName;
use serde::Deserialize;

use crate::{Error, read_ron};

#[derive(Debug, Builder)]
pub struct Scenario {
    name: String,
    description: String,
    deploy_a: Profile,
    deploy_b: Profile,
    a_start_spawn_zones: Vec<SpawnZoneName>,
    b_start_spawn_zones: Vec<SpawnZoneName>,
}

#[derive(Deserialize)]
struct ScenarioRaw {
    name: String,
    description: String,
    deploy_a: String,
    deploy_b: String,
    a_start_spawn_zones: Vec<SpawnZoneName>,
    b_start_spawn_zones: Vec<SpawnZoneName>,
}

impl Scenario {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn deploy_a(&self) -> &Profile {
        &self.deploy_a
    }

    pub fn deploy_b(&self) -> &Profile {
        &self.deploy_b
    }

    pub fn a_start_spawn_zones(&self) -> &[SpawnZoneName] {
        &self.a_start_spawn_zones
    }

    pub fn b_start_spawn_zones(&self) -> &[SpawnZoneName] {
        &self.b_start_spawn_zones
    }

    pub fn from_ron_file(path: impl AsRef<Path>) -> Result<Self, Error> {
        let path = path.as_ref();
        let dir = path.parent().unwrap_or(Path::new("."));
        let raw: ScenarioRaw = read_ron(path)?;

        Ok(Self {
            name: raw.name,
            description: raw.description,
            deploy_a: read_ron(&dir.join(raw.deploy_a))?,
            deploy_b: read_ron(&dir.join(raw.deploy_b))?,
            a_start_spawn_zones: raw.a_start_spawn_zones,
            b_start_spawn_zones: raw.b_start_spawn_zones,
        })
    }
}
