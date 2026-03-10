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
            Listening::Regions(regions) => regions
                .into_iter()
                .map(|tile| {
                    let region: WorldRegionIndex = tile.into();
                    self.regions_listeners[region.0 as usize].clone()
                })
                .collect::<Vec<_>>()
                .into_iter()
                .flatten()
                .collect::<FxHashSet<_>>(),
            Listening::Border(before, after) => {
                let before_listeners = self.find(Listening::Regions(vec![before]));
                let after_listeners = self.find(Listening::Regions(vec![after]));
                after_listeners
                    .into_iter()
                    .filter(|l| !before_listeners.contains(l))
                    .collect::<FxHashSet<_>>()
            }
        }
    }

    pub fn listener_regions(&self, listener: &T) -> &Vec<WorldRegionIndex> {
        static EMPTY: Vec<WorldRegionIndex> = vec![];
        self.listeners_regions.get(listener).unwrap_or(&EMPTY)
    }
}

#[derive(Debug, Clone)]
pub enum Listening {
    /// Will match with all listener of one of these regions
    Regions(Vec<WorldRegionIndex>),
    /// Will match with all listener NOT listening region& and listening regionb
    Border(WorldRegionIndex, WorldRegionIndex),
}
