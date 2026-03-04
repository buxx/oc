use bevy::prelude::*;
use oc_individual::IndividualIndex;
use oc_projectile::ProjectileId;
use rustc_hash::FxHashMap;

#[derive(Debug, Resource, Default)]
pub struct State {
    pub individuals: FxHashMap<IndividualIndex, Entity>,
    pub projectiles: FxHashMap<ProjectileId, Entity>,
}
