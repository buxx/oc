use oc_root::{WcfgFrom, WorldConfig};
use oc_utils::d2::Xy;
use rkyv::{Archive, Deserialize, Serialize};

use crate::region::RegionXy;

#[derive(Debug, Clone, Copy, Archive, Deserialize, Serialize, PartialEq)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct TileXy(pub Xy);

impl TileXy {
    pub fn clamped(&self, w: &WorldConfig) -> Self {
        Self(Xy(
            self.0.0.min(w.world_width - 1),
            self.0.1.min(w.world_height - 1),
        ))
    }

    pub fn point(&self, w: &WorldConfig) -> [f32; 2] {
        [
            self.0.0 as f32 * w.geo_pixels_per_tile as f32,
            self.0.1 as f32 * w.geo_pixels_per_tile as f32,
        ]
    }
}

impl From<TileXy> for (u64, u64) {
    fn from(value: TileXy) -> Self {
        (value.0.0, value.0.1)
    }
}

// a little bit tricky ...
impl WcfgFrom<TileXy> for [f32; 2] {
    fn from_(value: TileXy, w: &WorldConfig) -> Self {
        [
            value.0.0 as f32 * w.geo_pixels_per_tile as f32,
            value.0.1 as f32 * w.geo_pixels_per_tile as f32,
        ]
    }
}

// a little bit tricky ...
impl WcfgFrom<(f32, f32)> for TileXy {
    fn from_(value: (f32, f32), w: &WorldConfig) -> Self {
        TileXy(Xy(
            value.0 as u64 / w.geo_pixels_per_tile,
            value.1 as u64 / w.geo_pixels_per_tile,
        ))
    }
}

// a little bit tricky ...
impl WcfgFrom<TileXy> for [f32; 3] {
    fn from_(value: TileXy, w: &WorldConfig) -> Self {
        [
            value.0.0 as f32 * w.geo_pixels_per_tile as f32,
            value.0.1 as f32 * w.geo_pixels_per_tile as f32,
            0.0,
        ]
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    Archive,
    rkyv::Deserialize,
    rkyv::Serialize,
    serde::Serialize,
    serde::Deserialize,
    PartialEq,
    Eq,
    Hash,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct WorldTileIndex(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WorldHeightIndex(pub u64);

impl From<WorldHeightIndex> for WorldTileIndex {
    fn from(value: WorldHeightIndex) -> Self {
        Self(value.0)
    }
}

impl WcfgFrom<WorldHeightIndex> for WorldTileIndex {
    fn from_(value: WorldHeightIndex, _: &WorldConfig) -> Self {
        Self(value.0)
    }
}

impl WcfgFrom<WorldTileIndex> for WorldTileIndex {
    fn from_(value: WorldTileIndex, _: &WorldConfig) -> Self {
        Self(value.0)
    }
}

impl From<WorldTileIndex> for WorldHeightIndex {
    fn from(value: WorldTileIndex) -> Self {
        Self(value.0)
    }
}

impl WcfgFrom<WorldTileIndex> for WorldHeightIndex {
    fn from_(value: WorldTileIndex, _: &WorldConfig) -> Self {
        Self(value.0)
    }
}

impl WcfgFrom<WorldTileIndex> for TileXy {
    fn from_(WorldTileIndex(i): WorldTileIndex, w: &WorldConfig) -> Self {
        let x = i % w.world_width;
        let y = i / w.world_width;
        Self(Xy(x, y))
    }
}

impl WcfgFrom<WorldTileIndex> for Xy {
    fn from_(WorldTileIndex(i): WorldTileIndex, w: &WorldConfig) -> Self {
        let x = i % w.world_width;
        let y = i / w.world_width;
        Xy(x, y)
    }
}

impl WcfgFrom<WorldHeightIndex> for Xy {
    fn from_(WorldHeightIndex(i): WorldHeightIndex, w: &WorldConfig) -> Self {
        let x = i % w.world_width;
        let y = i / w.world_width;
        Xy(x, y)
    }
}

impl WcfgFrom<TileXy> for WorldTileIndex {
    fn from_(TileXy(Xy(x, y)): TileXy, w: &WorldConfig) -> Self {
        Self(y * w.world_width + x)
    }
}

impl WcfgFrom<[f32; 2]> for TileXy {
    fn from_(value: [f32; 2], w: &WorldConfig) -> Self {
        Self(Xy(
            value[0] as u64 / w.geo_pixels_per_tile,
            value[1] as u64 / w.geo_pixels_per_tile,
        ))
    }
}

impl WcfgFrom<[f32; 3]> for TileXy {
    fn from_(value: [f32; 3], w: &WorldConfig) -> Self {
        Self(Xy(
            value[0] as u64 / w.geo_pixels_per_tile,
            value[1] as u64 / w.geo_pixels_per_tile,
        ))
    }
}

impl WcfgFrom<RegionXy> for TileXy {
    fn from_(value: RegionXy, w: &WorldConfig) -> Self {
        let x = value.0.0 * w.region_width;
        let y = value.0.1 * w.region_height;
        Self(Xy(x, y))
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(999, 0, 999)]
    #[case(1000, 0, 1000)]
    #[case(1000, 1, 2000)]
    #[case(1001, 1, 2001)]
    pub fn test_world_tile_index(#[case] x: u64, #[case] y: u64, #[case] i: u64) {
        // Given

        use oc_root::physics::Meters;
        let w = WorldConfig::new(1000, 1000, Meters(0.1));

        // When
        let index = WorldTileIndex::from_(TileXy(Xy(x, y)), &w);

        // Then
        assert_eq!(index.0, i);
    }
}
