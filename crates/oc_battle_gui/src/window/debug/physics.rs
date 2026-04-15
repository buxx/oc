use derive_more::Constructor;
use oc_geo::region::Region;
use oc_geo::{Geo, region::RegionXy};
use oc_physics::Physic;

#[derive(Debug, Clone, Constructor)]
pub struct PhysicsRepr {
    pub position: [f32; 3],
    // pub tile: TileXy,
    pub region: RegionXy,
    // pub forces: Vec<Force>,
}

impl<T: Geo + Physic + Region> From<T> for PhysicsRepr {
    fn from(value: T) -> Self {
        Self {
            position: value.position().clone(),
            // tile: value.tile().clone(),
            region: value.region().clone().into(),
            // forces: value.forces().clone(),
        }
    }
}
