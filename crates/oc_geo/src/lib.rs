use crate::tile::TileXy;

pub mod region;
pub mod tile;
pub mod world;

pub trait Geo {
    fn tile(&self) -> &TileXy;
}

pub trait UpdateGeo {
    fn set_tile(&mut self, value: TileXy);
}
