use oc_geo::{region::RegionXy, tile::TileXy};
use rkyv::{Archive, Deserialize, Serialize};

use crate::Force;

#[derive(Debug, Clone, Archive, Deserialize, Serialize, PartialEq)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub enum Update {
    UpdatePosition([f32; 2]),
    UpdateTile(TileXy),
    UpdateRegion(RegionXy),
    PushForce(Force),
    RemoveForce(Force),
}
