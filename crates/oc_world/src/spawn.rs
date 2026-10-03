use oc_root::geo::WorldVec2;
#[cfg(feature = "bevy")]
use oc_root::{WcfgInto, WorldConfig};
use oc_utils::polygon::Polygon;

#[derive(
    Debug,
    rkyv::Archive,
    rkyv::Deserialize,
    serde::Deserialize,
    rkyv::Serialize,
    serde::Serialize,
    Clone,
    PartialEq,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct SpawnZoneName(pub String);

impl SpawnZoneName {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

#[cfg_attr(feature = "bevy", derive(bevy::prelude::Component))]
#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
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

#[cfg(feature = "bevy")]
impl SpawnZone {
    pub fn bevy_polygon(&self, w: &WorldConfig) -> geo::Polygon<f32> {
        let points: Vec<geo::Coord<f32>> = self
            .points
            .iter()
            .map(|p| {
                use oc_root::geo::ScreenVec2;

                let p: ScreenVec2 = (*p).into_(w);
                geo::Coord { x: p.x, y: p.y }
            })
            .collect();
        geo::Polygon::new(geo::LineString::new(points), vec![])
    }
}
