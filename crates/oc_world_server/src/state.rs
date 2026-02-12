use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use oc_world::World;

use crate::{index::Indexes, perf::Perf};

#[derive(Clone)]
pub struct State {
    pub perf: Arc<Perf>,
    world: Arc<RwLock<World>>,
    indexes: Arc<RwLock<Indexes>>,
}

impl State {
    pub fn new(world: World) -> Self {
        let perf = Arc::new(Perf::default());
        let indexes = Arc::new(RwLock::new(Indexes::new(&world)));
        let world = Arc::new(RwLock::new(world));

        Self {
            perf,
            world,
            indexes,
        }
    }

    pub fn world(&self) -> RwLockReadGuard<'_, World> {
        self.world.read().expect("Assum lock")
    }

    pub fn world_mut(&self) -> RwLockWriteGuard<'_, World> {
        self.world.write().expect("Assum lock")
    }

    pub fn _indexes(&self) -> RwLockReadGuard<'_, Indexes> {
        self.indexes.read().expect("Assum lock")
    }

    pub fn indexes_mut(&self) -> RwLockWriteGuard<'_, Indexes> {
        self.indexes.write().expect("Assum lock")
    }
}
