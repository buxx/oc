use derive_more::Constructor;
use oc_root::geo::WorldVec2;
use oc_utils::polygon::Polygon;

#[cfg_attr(feature = "bevy", derive(bevy::prelude::Component))]
#[derive(Clone, Debug, Constructor)]
pub struct Interior {
    pub id: u32,
    pub start: WorldVec2,
    pub points: Vec<WorldVec2>,
}

impl Polygon for Interior {
    fn points(&self) -> &[WorldVec2] {
        &self.points
    }
}
