use bevy::prelude::*;
use oc_projectile::{Projectile, ProjectileId};

#[derive(Debug, Event)]
pub struct InsertProjectileEvent(pub ProjectileId, pub Projectile);

#[derive(Debug, Event)]
pub struct UpdateProjectileEvent(pub ProjectileId, pub oc_projectile::Update);
