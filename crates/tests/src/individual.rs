use bon::Builder;
use oc_geo::{
    region::WorldRegionIndex,
    tile::{TileXy, WorldTileIndex},
};
use oc_individual::{
    Gesture, Individual,
    behavior::{Behavior, Intent},
};
use oc_root::{WcfgFrom, WorldConfig, geo::WorldVec3, side::Side};
use oc_utils::d2::Direction;

#[derive(Debug, Builder)]
pub struct TestIndividual {
    #[builder(default = Side::A)]
    side: Side,
    #[builder(default = WorldVec3::new(0., 0., 0.))]
    position: WorldVec3,
    #[builder(default = Behavior::Idle(Direction::NORTH))]
    behavior: Behavior,
    #[builder(default = Gesture::Idle(Direction::NORTH))]
    gesture: Gesture,
    #[builder(default = Intent::Idle(Direction::NORTH))]
    intent: Intent,
}

impl TestIndividual {
    pub fn make(self, w: &WorldConfig) -> Individual {
        let xy = TileXy::from_(self.position, w);
        let tile = WorldTileIndex::from_(xy, w);
        let region = WorldRegionIndex::from_(tile, w);

        Individual::fresh(self.side, self.position, tile, region)
            .with_gesture(self.gesture)
            .with_behavior(self.behavior)
            .with_intent(self.intent)
    }
}
