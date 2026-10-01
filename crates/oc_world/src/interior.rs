use derive_more::Constructor;
use oc_root::geo::WorldVec2;

#[derive(Clone, Debug, Constructor)]
pub struct Interior {
    pub id: u32,
    pub start: WorldVec2,
    pub points: Vec<WorldVec2>,
}
