use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use message_io::network::Endpoint;
use oc_geo::tile::{TileXy, WorldTileIndex};
use oc_individual::{Individual, IndividualIndex};
use oc_physics::{
    Physic,
    collision::{Material, Materials},
    volume::Volume,
};
use oc_projectile::Projectile;
#[cfg(feature = "debug")]
use oc_projectile::ProjectileId;
use oc_root::{GEO_PIXELS_PER_TILE, ids::Ids, tile::Tile};
use oc_utils::d2::Xy;
use oc_world::World;

use crate::{index::Indexes, perf::Perf, routing::Listeners};

#[derive(Clone)]
pub struct State {
    ids: Ids,
    pub perf: Arc<Perf>,
    world: Arc<RwLock<World>>,
    indexes: Arc<RwLock<Indexes>>,
    listeners: Arc<RwLock<Listeners<Endpoint>>>,
}

impl State {
    pub fn new(ids: Ids, world: World) -> Self {
        let perf = Arc::new(Perf::default());
        let indexes = Arc::new(RwLock::new(Indexes::new(&world)));
        let world = Arc::new(RwLock::new(world));
        let listeners = Arc::new(RwLock::new(Listeners::new()));

        Self {
            ids,
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

// impl From<Object<'_>> for ObjectId {
//     fn from(value: Object) -> Self {
//         match value {
//             Object::Individual(i, _) => ObjectId::Individual(i),
//             Object::Projectile(i, _) => ObjectId::Projectile(i),
//             Object::Tile(i, _) => ObjectId::Tile(i.into()),
//         }
//     }
// }

// impl From<IndividualIndex> for ObjectId {
//     fn from(value: IndividualIndex) -> Self {
//         Self::Individual(value)
//     }
// }

// impl From<ProjectileId> for ObjectId {
//     fn from(value: ProjectileId) -> Self {
//         Self::Projectile(value)
//     }
// }

// // TODO: move code
// #[derive(Debug)]
// pub enum Object<'a> {
//     Individual(IndividualIndex, &'a Individual),
//     Projectile(ProjectileId, &'a Projectile),
//     Tile(TileXy, Tile),
// }

// impl<'a> Physic for Object<'a> {
//     fn position(&self) -> &[f32; 2] {
//         todo!()
//     }

//     fn forces(&self) -> &Vec<oc_physics::Force> {
//         todo!()
//     }

//     fn volume(&self) -> &oc_physics::volume::Volume {
//         todo!()
//     }
// }

// impl<'a> Material for Object<'a> {
//     fn material(&self) -> oc_physics::collision::Materials {
//         todo!()
//     }
// }
