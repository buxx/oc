use bevy::prelude::*;
use oc_individual::behavior;

#[derive(Debug, Component)]
pub struct IndividualIndex(pub oc_individual::IndividualIndex);

#[derive(Debug, Component)]
pub struct Behavior(pub behavior::Behavior);
