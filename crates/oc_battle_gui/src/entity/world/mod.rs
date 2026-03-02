use bevy::prelude::*;
use oc_geo::tile::TileXy;

pub mod minimap;
pub mod region;

#[derive(Debug, Component)]
pub struct Tile(pub TileXy);

#[derive(Debug, Component)]
pub struct VisibleBattleSquare;

#[derive(Debug, Component)]
pub struct WorldMapBackground;
