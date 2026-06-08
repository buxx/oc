use std::{
    sync::{Arc, Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard},
    time::Instant,
};

#[cfg(feature = "perfs")]
use crate::perf::Perf;
use oc_geo::tile::WorldTileIndex;
use oc_individual::IndividualIndex;
use oc_mod::Mod;
use oc_projectile::ProjectileId;
use oc_root::{Client, WorldConfig, ids::Ids};
use oc_world::{World, load::WorldLoader, snapshot::Snapshot};
use serde::{Deserialize, Serialize};

use crate::{config::ServerConfig, index::Indexes, routing::Listeners, runner::update::Update};

#[derive(Clone)]
pub struct State<E: Client> {
    pub w: WorldConfig,
    pub _ids: Ids,
    pub _mod: Mod,
    #[cfg(feature = "perfs")]
    pub perf: Arc<Perf>,
    pub world: Arc<RwLock<World>>,
    pub indexes: Arc<RwLock<Indexes>>,
    pub listeners: Arc<RwLock<Listeners<E>>>,
    pub scheduled: Arc<Mutex<Vec<(Instant, Update)>>>,
}

impl<E: Client> State<E> {
    pub fn new(w: WorldConfig, ids: Ids, mod_: Mod, world: World) -> Self {
        #[cfg(feature = "perfs")]
        let perf = Arc::new(Perf::default());
        let indexes = Arc::new(RwLock::new(Indexes::new(&world)));
        let world = Arc::new(RwLock::new(world));
        let listeners = Arc::new(RwLock::new(Listeners::new(&w)));
        let scheduled = Arc::new(Mutex::new(vec![]));

        Self {
            w,
            _ids: ids,
            _mod: mod_,
            #[cfg(feature = "perfs")]
            perf,
            world,
            indexes,
            listeners,
            scheduled,
        }
    }

    pub fn _mod(&self) -> &Mod {
        &self._mod
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

    pub fn listeners(&self) -> RwLockReadGuard<'_, Listeners<E>> {
        self.listeners.read().expect("Assume lock")
    }

    pub fn listeners_mut(&self) -> RwLockWriteGuard<'_, Listeners<E>> {
        self.listeners.write().expect("Assume lock")
    }

    pub fn scheduled(&self) -> MutexGuard<'_, Vec<(Instant, Update)>> {
        self.scheduled.lock().expect("Assume lock")
    }
}

// TODO: move code
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObjectId {
    Individual(IndividualIndex),
    Projectile(ProjectileId),
    Tile(WorldTileIndex),
}

pub fn init<E: Client>(config: ServerConfig) -> Result<State<E>, anyhow::Error> {
    let cache = config.cache.clone();
    let world = config.world.clone();
    let mod_ = config.mod_.clone();

    let ids = Ids::default();
    let mod_ = Mod::load(&mod_, Some(&cache))?;
    let snapshot = Snapshot::load(&config.snapshot)?;
    let w = snapshot.w.clone();
    let world = WorldLoader::new(w.clone(), mod_.clone(), world.clone(), cache.clone());
    let world = world.load(&ids, snapshot)?;

    Ok(State::new(w.clone(), ids, mod_.clone(), world))
}
