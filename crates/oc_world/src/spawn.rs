use oc_root::geo::WorldVec2;
use oc_utils::polygon::Polygon;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct SpawnZoneName(pub String);

#[derive(Clone, Debug)]
pub struct SpawnZone {
    pub name: SpawnZoneName,
    pub start: WorldVec2,
    pub points: Vec<WorldVec2>,
}

impl Polygon for SpawnZone {
    fn points(&self) -> &[WorldVec2] {
        &self.points
    }
}
