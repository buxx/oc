use oc_geo::tile::WorldTileIndex;
use oc_root::WorldConfig;
use oc_world::{
    reader::MapReader,
    tile::{Nature, Tile},
};

pub trait TilesGenerator {
    fn tiles(&self, w: &WorldConfig) -> Vec<Tile>;
}

pub struct SameTileFiller(pub Nature);

impl TilesGenerator for SameTileFiller {
    fn tiles(&self, w: &WorldConfig) -> Vec<Tile> {
        (0..w.tiles_count)
            .map(|i| Tile::new(WorldTileIndex(i as u64), self.0, 0))
            .collect()
    }
}

impl TilesGenerator for MapReader {
    fn tiles(&self, w: &WorldConfig) -> Vec<Tile> {
        MapReader::tiles(self, w).unwrap() // TODO
    }
}
