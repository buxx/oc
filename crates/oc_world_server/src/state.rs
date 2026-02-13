use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use message_io::network::Endpoint;
use oc_world::World;

use crate::{index::Indexes, perf::Perf, routing::Listeners};

#[derive(Clone)]
pub struct State {
    pub perf: Arc<Perf>,
    world: Arc<RwLock<World>>,
    indexes: Arc<RwLock<Indexes>>,
    listeners: Arc<RwLock<Listeners<Endpoint>>>,
}

impl State {
    pub fn new(world: World) -> Self {
        let perf = Arc::new(Perf::default());
        let indexes = Arc::new(RwLock::new(Indexes::new(&world)));
        let world = Arc::new(RwLock::new(world));
        let listeners = Arc::new(RwLock::new(Listeners::new()));

        Self {
            perf,
            world,
            indexes,
            listeners,
        }
    }

    pub fn world(&self) -> RwLockReadGuard<'_, World> {
        self.world.read().expect("Assume lock")
    }

    pub fn world_mut(&self) -> RwLockWriteGuard<'_, World> {
        self.world.write().expect("Assume lock")
    }

    pub fn _indexes(&self) -> RwLockReadGuard<'_, Indexes> {
        self.indexes.read().expect("Assume lock")
    }

    pub fn indexes_mut(&self) -> RwLockWriteGuard<'_, Indexes> {
        self.indexes.write().expect("Assume lock")
    }

    pub fn listeners(&self) -> RwLockReadGuard<'_, Listeners<Endpoint>> {
        self.listeners.read().expect("Assume lock")
    }

    pub fn listeners_mut(&self) -> RwLockWriteGuard<'_, Listeners<Endpoint>> {
        self.listeners.write().expect("Assume lock")
    }
}
