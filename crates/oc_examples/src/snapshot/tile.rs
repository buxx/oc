use oc_geo::tile::WorldTileIndex;
use oc_root::TILES_COUNT;
use oc_world::{
    reader::MapReader,
    tile::{Nature, Tile},
};

pub trait TilesGenerator {
    fn tiles(&self) -> Vec<Tile>;
}

pub struct SameTileFiller(pub Nature);

impl TilesGenerator for SameTileFiller {
    fn tiles(&self) -> Vec<Tile> {
        (0..TILES_COUNT)
            .map(|i| Tile::new(WorldTileIndex(i as u64), self.0, 0))
            .collect()
    }
}

impl TilesGenerator for MapReader {
    fn tiles(&self) -> Vec<Tile> {
        MapReader::tiles(self).unwrap() // TODO
    }
}
