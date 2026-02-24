use oc_geo::region::WorldRegionIndex;
use oc_root::REGIONS_COUNT;
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
        self.all.retain(|listener_| listener_ != listener);

        let regions = self.listener_regions(listener).clone();
        for region in regions {
            self.forgot_region(listener, region)
        }
    }

    pub fn listen_region(&mut self, listener: &T, region: WorldRegionIndex) {
        tracing::trace!(name = "listeners-listen-region", region = ?region);
        self.regions_listeners[region.0 as usize].push(listener.clone());
        self.listeners_regions
            .entry(listener.clone())
            .or_default()
            .push(region);
    }

    pub fn forgot_region(&mut self, listener: &T, region: WorldRegionIndex) {
        self.regions_listeners[region.0 as usize].retain(|l| l != listener);
    }

    pub fn find(&self, filter: Listening) -> FxHashSet<T> {
        match filter {
            Listening::Regions(tiles) => tiles
                .into_iter()
                .map(|tile| {
                    let region: WorldRegionIndex = tile.into();
                    self.regions_listeners[region.0 as usize].clone()
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

    // #[cfg(test)]
    // pub fn regions_listeners(&self) -> &[Vec<T>] {
    //     &self.regions_listeners
    // }

    // #[cfg(test)]
    // pub fn listeners_regions(&self) -> &FxHashMap<T, Vec<WorldRegionIndex>> {
    //     &self.listeners_regions
    // }
}

#[derive(Debug, Clone)]
pub enum Listening {
    Regions(Vec<WorldRegionIndex>),
}

// #[derive(Debug, Clone)]
// pub enum Listen {
//     Area(TileXy, TileXy),
// }

// #[cfg(test)]
// mod tests {
//     use oc_root::{WORLD_HEIGHT, WORLD_WIDTH};

//     use super::*;

//     #[test]
//     fn test_listen_area() {
//         // Given
//         let mut listener = Listeners::new();
//         let from = TileXy(Xy(0, 0));
//         let to = TileXy(Xy(WORLD_WIDTH as u64 - 1, WORLD_HEIGHT as u64 - 1)); // Whole map is listened
//         let filter = Listen::Area(from, to);

//         // When (listen all regions)
//         listener.listen((), filter);

//         // Then (all regions listened in `regions_listeners` and `listeners_regions`)
//         let expected = vec![vec![()]; REGIONS_COUNT]; // All region is listened
//         assert_eq!(listener.regions_listeners(), expected);
//         let expected = (0..REGIONS_COUNT).map(WorldRegionIndex).collect();
//         assert_eq!(listener.listeners_regions().get(&()), Some(&expected));

//         // When (listen only first region)
//         let filter = Listen::Area(TileXy(Xy(0, 0)), TileXy(Xy(0, 0))); // No longer listen others
//         listener.listen((), filter);

//         // Then (`regions_listeners` and `listeners_regions` no longer list previons regions)
//         let expected = vec![vec![()]];
//         assert_eq!(listener.regions_listeners(), expected);
//         let expected = Some(&vec![WorldRegionIndex(0)]);
//         assert_eq!(listener.listeners_regions().get(&()), expected);
//     }
// }
