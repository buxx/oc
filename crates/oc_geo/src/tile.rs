use oc_root::{GEO_PIXELS_PER_TILE, REGION_HEIGHT, REGION_WIDTH, WORLD_HEIGHT, WORLD_WIDTH};
use oc_utils::d2::Xy;
use rkyv::{Archive, Deserialize, Serialize};

use crate::region::RegionXy;

#[derive(Debug, Clone, Copy, Archive, Deserialize, Serialize, PartialEq)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct TileXy(pub Xy);

impl TileXy {
    pub fn clamped(&self) -> Self {
        Self(Xy(
            self.0.0.min(WORLD_WIDTH as u64 - 1),
            self.0.1.min(WORLD_HEIGHT as u64 - 1),
        ))
    }

    pub fn point(&self) -> [f32; 2] {
        [
            self.0.0 as f32 * GEO_PIXELS_PER_TILE as f32,
            self.0.1 as f32 * GEO_PIXELS_PER_TILE as f32,
        ]
    }
}

impl From<TileXy> for (u64, u64) {
    fn from(value: TileXy) -> Self {
        (value.0.0, value.0.1)
    }
}

// a little bit tricky ...
impl From<TileXy> for [f32; 2] {
    fn from(value: TileXy) -> Self {
        [
            value.0.0 as f32 * GEO_PIXELS_PER_TILE as f32,
            value.0.1 as f32 * GEO_PIXELS_PER_TILE as f32,
        ]
    }
}

// a little bit tricky ...
impl From<(f32, f32)> for TileXy {
    fn from(value: (f32, f32)) -> Self {
        TileXy(Xy(
            value.0 as u64 / GEO_PIXELS_PER_TILE,
            value.1 as u64 / GEO_PIXELS_PER_TILE,
        ))
    }
}

// a little bit tricky ...
impl From<TileXy> for [f32; 3] {
    fn from(value: TileXy) -> Self {
        [
            value.0.0 as f32 * GEO_PIXELS_PER_TILE as f32,
            value.0.1 as f32 * GEO_PIXELS_PER_TILE as f32,
            0.0,
        ]
    }
}

#[derive(Debug, Clone, Copy, Archive, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct WorldTileIndex(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WorldHeightIndex(pub u64);

impl From<WorldHeightIndex> for WorldTileIndex {
    fn from(value: WorldHeightIndex) -> Self {
        Self(value.0)
    }
}

impl From<WorldTileIndex> for WorldHeightIndex {
    fn from(value: WorldTileIndex) -> Self {
        Self(value.0)
    }
}

impl From<WorldTileIndex> for TileXy {
    fn from(WorldTileIndex(i): WorldTileIndex) -> Self {
        let x = i % WORLD_WIDTH as u64;
        let y = i / WORLD_WIDTH as u64;
        Self(Xy(x, y))
    }
}

impl From<WorldTileIndex> for Xy {
    fn from(WorldTileIndex(i): WorldTileIndex) -> Self {
        let x = i % WORLD_WIDTH as u64;
        let y = i / WORLD_WIDTH as u64;
        Xy(x, y)
    }
}

impl From<WorldHeightIndex> for Xy {
    fn from(WorldHeightIndex(i): WorldHeightIndex) -> Self {
        let x = i % WORLD_WIDTH as u64;
        let y = i / WORLD_WIDTH as u64;
        Xy(x, y)
    }
}

impl From<TileXy> for WorldTileIndex {
    fn from(TileXy(Xy(x, y)): TileXy) -> Self {
        Self(y * WORLD_WIDTH as u64 + x)
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

impl From<[f32; 3]> for TileXy {
    fn from(value: [f32; 3]) -> Self {
        Self(Xy(
            value[0] as u64 / GEO_PIXELS_PER_TILE,
            value[1] as u64 / GEO_PIXELS_PER_TILE,
        ))
    }
}

impl From<RegionXy> for TileXy {
    fn from(value: RegionXy) -> Self {
        let x = value.0.0 * REGION_WIDTH as u64;
        let y = value.0.1 * REGION_HEIGHT as u64;
        Self(Xy(x, y))
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
        assert_eq!(index.0, i as u64);
    }
}
