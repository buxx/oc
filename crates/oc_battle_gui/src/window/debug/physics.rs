use derive_more::Constructor;
use oc_geo::region::Region;
use oc_geo::{Geo, region::RegionXy};
use oc_physics::Physic;
use oc_root::{WcfgFrom, WcfgInto, WorldConfig};

#[derive(Debug, Clone, Constructor)]
pub struct PhysicsRepr {
    pub position: [f32; 3],
    // pub tile: TileXy,
    pub region: RegionXy,
    // pub forces: Vec<Force>,
}

impl<T: Geo + Physic + Region> WcfgFrom<T> for PhysicsRepr {
    fn from_(value: T, w: &WorldConfig) -> Self {
        Self {
            position: value.position(w).clone(),
            // tile: value.tile().clone(),
            region: value.region().clone().into_(w),
            // forces: value.forces().clone(),
        }
    }
}
