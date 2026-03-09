use oc_geo::{region::RegionXy, tile::TileXy};
use rkyv::{Archive, Deserialize, Serialize};

use crate::Force;

#[cfg(feature = "bevy")]
pub mod bevy;

#[derive(Debug, Clone, Archive, Deserialize, Serialize, PartialEq)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub enum Update {
    SetPosition([f32; 2]),
    SetTile(TileXy),
    SetRegion(RegionXy),
    PushForce(Force),
    RemoveForce(Force),
}
