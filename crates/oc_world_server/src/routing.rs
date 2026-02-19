use oc_geo::{
    region::{RegionXy, WorldRegionIndex},
    tile::TileXy,
};
use oc_root::REGIONS_COUNT;
use oc_utils::d2::Xy;
use rustc_hash::{FxHashMap, FxHashSet};
use std::hash::Hash;

pub struct Listeners<T: Clone + PartialEq + Hash + std::cmp::Eq> {
    // All endpoints currently listening something (aka all clients)
    all: Vec<T>,
    // Endpoints listening specifics regions (level 1 vector is all regions vector)
    regions_listeners: Vec<Vec<T>>,
    // Which regions listen
    listeners_regions: FxHashMap<T, Vec<WorldRegionIndex>>,
}

impl<T: Clone + PartialEq + Hash + std::cmp::Eq> Listeners<T> {
    pub fn new() -> Self {
        Self {
            all: vec![],
            regions_listeners: vec![vec![]; REGIONS_COUNT],
            listeners_regions: FxHashMap::default(),
        }
    }

    pub fn push(&mut self, endpoint: T) {
        self.all.push(endpoint)
    }

    // TODO: test me
    pub fn remove(&mut self, listener: &T) {
        self.all.retain(|endpoint_| endpoint_ != listener);
        if let Some(regions) = self.listeners_regions.get(listener) {
            for region in regions {
                self.regions_listeners[region.0].retain(|l| l != listener);
            }
            self.listeners_regions.remove(&listener);
        }
    }

    pub fn listen(&mut self, endpoint: T, filter: Listen) {
        match filter {
            Listen::Area(from, to) => {
                let from_region_xy: RegionXy = from.into();
                let to_region_xy: RegionXy = to.into();

                let (from_region_x, from_region_y) = (from_region_xy.0.0, from_region_xy.0.1);
                let (to_region_x, to_region_y) = (to_region_xy.0.0, to_region_xy.0.1);

                for x in from_region_x..=to_region_x {
                    for y in from_region_y..=to_region_y {
                        let region: WorldRegionIndex = RegionXy(Xy(x, y)).into();
                        tracing::trace!(name = "listeners-listen-region", region = ?region);
                        self.regions_listeners[region.0].push(endpoint.clone());
                        self.listeners_regions
                            .entry(endpoint.clone())
                            .or_default()
                            .push(region);
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
                    self.regions_listeners[region.0].clone()
                })
                .collect::<Vec<_>>()
                .into_iter()
                .flatten()
                .collect::<FxHashSet<_>>(),
        }
    }

    pub fn listener_regions(&self, listener: &T) -> &Vec<WorldRegionIndex> {
        static EMPTY: Vec<WorldRegionIndex> = vec![];
        self.listeners_regions.get(listener).unwrap_or(&EMPTY)
    }

    #[cfg(test)]
    pub fn regions_listeners(&self) -> &[Vec<T>] {
        &self.regions_listeners
    }

    #[cfg(test)]
    pub fn listeners_regions(&self) -> &FxHashMap<T, Vec<WorldRegionIndex>> {
        &self.listeners_regions
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

    use super::*;

    #[test]
    fn test_listen_area() {
        // Given
        let mut listener = Listeners::new();
        let from = TileXy(Xy(0, 0));
        let to = TileXy(Xy(WORLD_WIDTH as u64 - 1, WORLD_HEIGHT as u64 - 1)); // Whole map is listened
        let filter = Listen::Area(from, to);

        // When
        listener.listen((), filter);

        // Then
        let expected = vec![vec![()]; REGIONS_COUNT]; // All region is listened
        assert_eq!(listener.regions_listeners(), expected);
        let expected = (0..REGIONS_COUNT).map(WorldRegionIndex).collect();
        assert_eq!(listener.listeners_regions().get(&()), Some(&expected));
    }
}
