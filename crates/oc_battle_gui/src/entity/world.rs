use bevy::prelude::*;
use oc_geo::{region::WorldRegionIndex, tile::TileXy};

#[derive(Debug, Component)]
pub struct Tile(pub TileXy);

#[derive(Debug, Component)]
pub struct Region(pub WorldRegionIndex);
