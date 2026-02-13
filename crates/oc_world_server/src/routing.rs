use oc_geo::{
    region::{RegionXy, WorldRegionIndex},
    tile::TileXy,
};
use oc_root::REGIONS_COUNT;
use oc_utils::d2::Xy;
use rustc_hash::FxHashSet;
use std::hash::Hash;

pub struct Listeners<T: Clone + PartialEq + Hash + std::cmp::Eq> {
    all: Vec<T>,
    regions: Vec<Vec<T>>,
}

impl<T: Clone + PartialEq + Hash + std::cmp::Eq> Listeners<T> {
    pub fn new() -> Self {
        Self {
            all: vec![],
            regions: vec![vec![]; REGIONS_COUNT],
        }
    }

    pub fn push(&mut self, endpoint: T) {
        self.all.push(endpoint)
    }

    pub fn remove(&mut self, endpoint: T) {
        self.all.retain(|endpoint_| endpoint_ != &endpoint);
        // TODO: remove from .regions
    }

    pub fn listen(&mut self, endpoint: T, filter: Listen) {
        match filter {
            Listen::Area(from, to) => {
                let from_region_xy: RegionXy = from.into();
                let to_region_xy: RegionXy = to.into();

                for x in (from_region_xy.0.0)..=(to_region_xy.0.0) {
                    for y in (from_region_xy.0.1)..=(to_region_xy.0.1) {
                        let region: WorldRegionIndex = RegionXy(Xy(x, y)).into();
                        tracing::trace!(name = "listeners-listen-region", region = ?region);
                        self.regions[region.0].push(endpoint.clone())
                    }
                }
            }
        }
    }

    pub fn find(&self, filter: Listening) -> FxHashSet<T> {
        match filter {
            Listening::TileXy(tiles) => tiles
                .into_iter()
                .map(|tile| {
                    let region: WorldRegionIndex = tile.into();
                    self.regions[region.0].clone()
                })
                .collect::<Vec<_>>()
                .into_iter()
                .flatten()
                .collect::<FxHashSet<_>>(),
        }
    }

    #[cfg(test)]
    pub fn regions(&self) -> &[Vec<T>] {
        &self.regions
    }
}

#[derive(Debug, Clone)]
pub enum Listening {
    TileXy(Vec<TileXy>),
}

#[derive(Debug, Clone)]
pub enum Listen {
    Area(TileXy, TileXy),
}

#[cfg(test)]
mod tests {
    use oc_root::{WORLD_HEIGHT, WORLD_WIDTH};
    use rstest::rstest;

    use super::*;

    #[rstest]
    fn test_listen_area() {
        // Given
        let mut listener = Listeners::new();
        let from = TileXy(Xy(0, 0));
        let to = TileXy(Xy(WORLD_WIDTH as u64, WORLD_HEIGHT as u64)); // Whole map is listened
        let filter = Listen::Area(from, to);

        // When
        listener.listen((), filter);

        // Then
        let expected = vec![vec![()]; REGIONS_COUNT]; // All region is listened
        assert_eq!(listener.regions(), expected)
    }
}
