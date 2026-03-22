use oc_geo::{region::RegionXy, tile::TileXy};
use rkyv::{Archive, Deserialize, Serialize};

use crate::{Force, volume::Volume};

#[cfg(feature = "bevy")]
pub mod bevy;

#[derive(Debug, Clone, Archive, Deserialize, Serialize, PartialEq)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub enum Update {
    SetPosition([f32; 3], [f32; 3]), // new, before
    SetTile(TileXy, TileXy),         // new, before
    SetRegion(RegionXy, RegionXy),   // new, before
    PushForce(Force),
    RemoveForce(Force),
    SetVolume(Volume, Volume), // new, before
}
