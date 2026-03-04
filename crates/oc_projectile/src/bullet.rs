use derive_more::Constructor;
use oc_geo::region::RegionXy;
use oc_geo::tile::TileXy;
use oc_physics::Force;
use oc_physics::Physic;
use oc_physics::collision::Material;
use oc_physics::collision::Materials;
use oc_utils::d2::Xy;
use rkyv::{Archive, Deserialize, Serialize};

#[derive(Archive, Deserialize, Serialize, Clone, Copy, Debug, PartialEq, Eq, Constructor, Hash)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct BulletId(pub u64);

#[derive(Archive, Deserialize, Serialize, Debug, PartialEq, Constructor, Clone)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct Bullet {
    pub position: [f32; 2],
    pub tile: TileXy,
    pub region: RegionXy,
    pub forces: Vec<Force>,
}

impl Physic for &Bullet {
    fn position(&self) -> &[f32; 2] {
        &self.position
    }

    fn xy(&self) -> &Xy {
        &self.tile.0
    }

    fn forces(&self) -> &Vec<Force> {
        &self.forces
    }
}

impl Material for &Bullet {
    fn material(&self) -> Materials {
        Materials::Traversable
    }
}
