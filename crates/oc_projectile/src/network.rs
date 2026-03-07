use derive_more::Deref;
use rkyv::{Archive, Deserialize, Serialize};

use crate::{Projectile as Projectile_, ProjectileId};

#[derive(Debug, Clone, Archive, Deserialize, Serialize, PartialEq)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub enum Projectile {
    Insert(ProjectileId, Projectile_),
    Physics(ProjectileId, oc_physics::update::Update),
}

#[derive(Debug, Clone, Archive, Deserialize, Serialize, PartialEq, Deref)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct SpawnProjectile(pub Projectile_);
