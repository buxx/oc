use bevy::prelude::*;
use oc_individual::IndividualIndex;
use rustc_hash::FxHashMap;

#[derive(Debug, Resource, Default)]
pub struct State {
    pub individuals: FxHashMap<IndividualIndex, Entity>,
}
