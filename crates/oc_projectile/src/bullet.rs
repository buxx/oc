use oc_geo::tile::TileXy;
use oc_geo::{region::WorldRegionIndex, tile::WorldTileIndex};
#[cfg(feature = "debug")]
use oc_individual::IndividualIndex;
use oc_physics::Force;
use oc_root::geo::WorldVec3;
use oc_root::side::Side;
use oc_root::{WcfgInto, WorldConfig};
use rkyv::{Archive, Deserialize, Serialize};

#[derive(Archive, Deserialize, Serialize, Clone, Debug, PartialEq)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct Bullet {
    pub position: WorldVec3,
    pub side: Side,
    pub tile: WorldTileIndex,
    pub region: WorldRegionIndex,
    pub forces: Vec<Force>,
    #[cfg(feature = "debug")]
    pub shooter: Option<IndividualIndex>,
}

impl Bullet {
    pub fn new(
        position: WorldVec3,
        side: Side,
        forces: Vec<Force>,
        #[cfg(feature = "debug")] shooter: Option<IndividualIndex>,
        w: &WorldConfig,
    ) -> Self {
        let tile: TileXy = position.into_(w);
        let tile: WorldTileIndex = tile.into_(w);
        let region: WorldRegionIndex = tile.into_(w);

        Self {
            position,
            side,
            tile,
            region,
            forces,
            #[cfg(feature = "debug")]
            shooter,
        }
    }
}
