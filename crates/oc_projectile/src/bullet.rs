use oc_geo::tile::TileXy;
use oc_geo::{region::WorldRegionIndex, tile::WorldTileIndex};
use oc_physics::Force;
use oc_root::{WcfgInto, WorldConfig};
use rkyv::{Archive, Deserialize, Serialize};

#[derive(Archive, Deserialize, Serialize, Clone, Debug, PartialEq)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct Bullet {
    pub position: [f32; 3],
    pub tile: WorldTileIndex,
    pub region: WorldRegionIndex,
    pub forces: Vec<Force>,
}

impl Bullet {
    pub fn new(position: [f32; 3], forces: Vec<Force>, w: &WorldConfig) -> Self {
        let tile: TileXy = position.into_(w);
        let tile: WorldTileIndex = tile.into_(w);
        let region: WorldRegionIndex = tile.into_(w);

        Self {
            position,
            tile,
            region,
            forces,
        }
    }
}
