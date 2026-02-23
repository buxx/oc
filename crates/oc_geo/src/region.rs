use oc_root::{REGION_HEIGHT, REGION_WIDTH, REGIONS_WIDTH};
use oc_utils::d2::Xy;
use rkyv::{Archive, Deserialize, Serialize};

use crate::tile::{TileXy, WorldTileIndex};

#[derive(Debug, Clone, Copy, Archive, Deserialize, Serialize, PartialEq)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct RegionXy(pub Xy);

impl From<RegionXy> for (u64, u64) {
    fn from(value: RegionXy) -> Self {
        (value.0.0, value.0.1)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Archive, Deserialize, Serialize)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct WorldRegionIndex(pub u64);

impl From<WorldRegionIndex> for RegionXy {
    fn from(WorldRegionIndex(i): WorldRegionIndex) -> Self {
        let x = i % REGIONS_WIDTH as u64;
        let y = i / REGIONS_WIDTH as u64;
        Self(Xy(x as u64, y as u64))
    }
}

impl From<WorldRegionIndex> for Xy {
    fn from(WorldRegionIndex(i): WorldRegionIndex) -> Self {
        let x = i % REGIONS_WIDTH as u64;
        let y = i / REGIONS_WIDTH as u64;
        Xy(x as u64, y as u64)
    }
}

impl From<RegionXy> for WorldRegionIndex {
    fn from(RegionXy(Xy(x, y)): RegionXy) -> Self {
        Self(y * REGIONS_WIDTH as u64 + x)
    }
}

impl From<WorldTileIndex> for WorldRegionIndex {
    fn from(tile: WorldTileIndex) -> Self {
        let tile_xy: TileXy = tile.into();
        let region_xy: RegionXy = tile_xy.into();
        region_xy.into()
    }
}

impl From<TileXy> for RegionXy {
    fn from(value: TileXy) -> Self {
        Self(Xy(
            value.0.0 / REGION_WIDTH as u64,
            value.0.1 / REGION_HEIGHT as u64,
        ))
    }
}

impl From<TileXy> for WorldRegionIndex {
    fn from(tile: TileXy) -> Self {
        let region_xy: RegionXy = tile.into();
        region_xy.into()
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(REGIONS_WIDTH as u64 - 1, 0, REGIONS_WIDTH as u64 - 1)]
    #[case(REGIONS_WIDTH as u64, 0, REGIONS_WIDTH as u64)]
    #[case(REGIONS_WIDTH as u64, 1, REGIONS_WIDTH as u64 * 2)]
    #[case(REGIONS_WIDTH as u64 + 1, 1, REGIONS_WIDTH as u64 * 2 + 1)]
    pub fn test_world_region_index_from_world_region_xy(
        #[case] x: u64,
        #[case] y: u64,
        #[case] i: u64,
    ) {
        // When
        let index = WorldRegionIndex::from(RegionXy(Xy(x as u64, y)));

        // Then
        assert_eq!(index.0, i);
    }

    #[rstest]
    #[case(0, 0, 0)] // Top left tile is de facto in first region
    #[case(REGION_WIDTH as u64, 0, 1)] // First tile in second region
    #[case(0, 1, 0)] // First tile in second world row is first region of world
    #[case(0, REGION_HEIGHT as u64, REGIONS_WIDTH as u64)]
    pub fn test_world_region_index_from_world_tile_index(
        #[case] x: u64,
        #[case] y: u64,
        #[case] i: u64,
    ) {
        // When
        let tile_index = WorldTileIndex::from(TileXy(Xy(x as u64, y)));
        let region_index = WorldRegionIndex::from(tile_index);

        // Then
        assert_eq!(region_index.0, i);
    }
}
