use bevy::prelude::*;
use oc_geo::tile::TileXy;

pub mod region;

#[derive(Debug, Component)]
pub struct Tile(pub TileXy);
