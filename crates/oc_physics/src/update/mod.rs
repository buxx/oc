use oc_geo::{region::WorldRegionIndex, tile::WorldTileIndex};
use oc_mod::nature::Traversability;
use oc_root::geo::WorldVec3;
use rkyv::{Archive, Deserialize, Serialize};

use crate::{Force, volume::Volume};

#[cfg(feature = "bevy")]
pub mod bevy;

#[derive(Debug, Clone, Archive, Deserialize, Serialize, PartialEq)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub enum Update {
    SetPosition(WorldVec3, WorldVec3),             // new, before
    SetTile(WorldTileIndex, WorldTileIndex),       // new, before
    SetRegion(WorldRegionIndex, WorldRegionIndex), // new, before
    PushForce(Force),
    RemoveForce(Force),
    SetVolumes(Vec<(Volume, Traversability)>, Vec<(Volume, Traversability)>), // new, before
}
