use bevy::prelude::*;
use oc_geo::region::WorldRegionIndex;

#[cfg(feature = "debug")]
pub mod debug;

#[derive(Debug, Event)]
pub struct ListeningRegion(pub WorldRegionIndex);

#[derive(Debug, Event)]
pub struct ForgottenRegion(pub WorldRegionIndex);
