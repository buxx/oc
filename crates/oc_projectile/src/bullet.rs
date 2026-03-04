use oc_geo::region::RegionXy;
use oc_geo::tile::TileXy;
use oc_physics::Force;
use rkyv::{Archive, Deserialize, Serialize};

#[derive(Archive, Deserialize, Serialize, Clone, Debug, PartialEq)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct Bullet {
    pub position: [f32; 2],
    pub tile: TileXy,
    pub region: RegionXy,
    pub forces: Vec<Force>,
}

impl Bullet {
    pub fn new(position: [f32; 2], forces: Vec<Force>) -> Self {
        let tile: TileXy = position.into();
        let region: RegionXy = tile.into();

        Self {
            position,
            tile,
            region,
            forces,
        }
    }
}
