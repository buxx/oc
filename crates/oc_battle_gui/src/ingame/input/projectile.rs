use bevy::prelude::*;
use oc_projectile::{Projectile, ProjectileId};

#[derive(Debug, Event)]
pub struct InsertProjectileEvent(pub ProjectileId, pub Projectile);

#[derive(Debug, Event)]
pub struct UpdateProjectilePhysicsEvent(pub ProjectileId, pub oc_physics::update::Update);
