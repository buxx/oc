use std::{
    sync::{Arc, Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard},
    time::Instant,
};

use message_io::network::Endpoint;
use oc_geo::tile::WorldTileIndex;
use oc_individual::IndividualIndex;
use oc_projectile::ProjectileId;
#[cfg(feature = "debug")]
use oc_root::ids::Ids;
use oc_world::World;

use crate::{index::Indexes, perf::Perf, routing::Listeners, runner::update::Update};

#[derive(Clone)]
pub struct State {
    #[cfg(feature = "debug")]
    ids: Ids,
    pub perf: Arc<Perf>,
    world: Arc<RwLock<World>>,
    indexes: Arc<RwLock<Indexes>>,
    listeners: Arc<RwLock<Listeners<Endpoint>>>,
    scheduled: Arc<Mutex<Vec<(Instant, Update)>>>,
}

impl State {
    pub fn new(#[cfg(feature = "debug")] ids: Ids, world: World) -> Self {
        let perf = Arc::new(Perf::default());
        let indexes = Arc::new(RwLock::new(Indexes::new(&world)));
        let world = Arc::new(RwLock::new(world));
        let listeners = Arc::new(RwLock::new(Listeners::new()));
        let scheduled = Arc::new(Mutex::new(vec![]));

        Self {
            #[cfg(feature = "debug")]
            ids,
            perf,
            world,
            indexes,
            listeners,
            scheduled,
        }
    }

    pub fn world(&self) -> RwLockReadGuard<'_, World> {
        self.world.read().expect("Assume lock")
    }

    pub fn world_mut(&self) -> RwLockWriteGuard<'_, World> {
        self.world.write().expect("Assume lock")
    }

    pub fn indexes(&self) -> RwLockReadGuard<'_, Indexes> {
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

    pub fn scheduled(&self) -> MutexGuard<'_, Vec<(Instant, Update)>> {
        self.scheduled.lock().expect("Assume lock")
    }

    #[cfg(feature = "debug")]
    pub fn new_projectile_id(&self) -> ProjectileId {
        let projectiles = &self.ids.projectiles;
        let id = projectiles.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        ProjectileId(id)
    }
}

// TODO: move code
#[derive(Debug, Clone)]
pub enum ObjectId {
    Individual(IndividualIndex),
    Projectile(ProjectileId),
    Tile(WorldTileIndex),
}
