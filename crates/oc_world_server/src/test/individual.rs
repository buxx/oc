use bon::Builder;
use glam::Vec3;
use oc_geo::{
    region::WorldRegionIndex,
    tile::{TileXy, WorldTileIndex},
};
use oc_individual::Individual;
use oc_root::{WcfgFrom, WorldConfig, side::Side};
use oc_utils::d2::Xy;

#[derive(Debug, Builder)]
pub struct TestIndividual {
    i: u64,
    #[builder(default = Side::A)]
    side: Side,
    #[builder(default = Vec3::new(0., 0., 0.))]
    position: Vec3,
}

impl TestIndividual {
    pub fn make(self, w: &WorldConfig) -> Individual {
        let xy = TileXy(Xy::from(self.position));
        let tile = WorldTileIndex::from_(xy, w);
        let region = WorldRegionIndex::from_(tile, w);
        Individual::fresh(self.side, self.position, tile, region)
    }
}
