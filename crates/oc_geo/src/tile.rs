use oc_root::{GEO_PIXELS_PER_TILE, WORLD_HEIGHT, WORLD_WIDTH};
use oc_utils::d2::Xy;
use rkyv::{Archive, Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Archive, Deserialize, Serialize, PartialEq)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct TileXy(pub Xy);

impl TileXy {
    pub fn resize(&self) -> Self {
        Self(Xy(
            self.0.0.max(0).min(WORLD_WIDTH as u64 - 1),
            self.0.1.max(0).min(WORLD_HEIGHT as u64 - 1),
        ))
    }
}

impl From<TileXy> for (u64, u64) {
    fn from(value: TileXy) -> Self {
        (value.0.0, value.0.1)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct WorldTileIndex(pub usize);

impl From<WorldTileIndex> for TileXy {
    fn from(WorldTileIndex(i): WorldTileIndex) -> Self {
        let x = i % WORLD_WIDTH;
        let y = i / WORLD_WIDTH;
        Self(Xy(x as u64, y as u64))
    }
}

impl From<WorldTileIndex> for Xy {
    fn from(WorldTileIndex(i): WorldTileIndex) -> Self {
        let x = i % WORLD_WIDTH;
        let y = i / WORLD_WIDTH;
        Xy(x as u64, y as u64)
    }
}

impl From<TileXy> for WorldTileIndex {
    fn from(TileXy(Xy(x, y)): TileXy) -> Self {
        Self(y as usize * WORLD_WIDTH + x as usize)
    }
}

impl From<[f32; 2]> for TileXy {
    fn from(value: [f32; 2]) -> Self {
        Self(Xy(
            value[0] as u64 / GEO_PIXELS_PER_TILE,
            value[1] as u64 / GEO_PIXELS_PER_TILE,
        ))
    }
}

#[cfg(test)]
mod tests {
    use oc_root::WORLD_WIDTH;
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(WORLD_WIDTH as u64 - 1, 0, WORLD_WIDTH - 1)]
    #[case(WORLD_WIDTH as u64, 0, WORLD_WIDTH)]
    #[case(WORLD_WIDTH as u64, 1, WORLD_WIDTH * 2)]
    #[case(WORLD_WIDTH as u64 + 1, 1, WORLD_WIDTH * 2 + 1)]
    pub fn test_world_tile_index(#[case] x: u64, #[case] y: u64, #[case] i: usize) {
        // When
        let index = WorldTileIndex::from(TileXy(Xy(x as u64, y)));

        // Then
        assert_eq!(index.0, i);
    }
}
