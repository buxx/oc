use std::hash::Hash;

use bevy::prelude::*;
use rustc_hash::FxHashMap;

#[derive(Debug, Resource, Deref, DerefMut)]
pub struct EntityMapping<I: Hash>(pub FxHashMap<I, Entity>);

impl<I: Hash> Default for EntityMapping<I> {
    fn default() -> Self {
        Self(FxHashMap::default())
    }
}
