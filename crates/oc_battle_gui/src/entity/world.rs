use bevy::prelude::*;
use oc_geo::tile::TileXy;

#[derive(Debug, Component)]
pub struct Tile(pub TileXy);
