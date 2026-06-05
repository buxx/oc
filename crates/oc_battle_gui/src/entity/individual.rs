use bevy::prelude::*;
use oc_individual::behavior;

#[derive(Debug, Component)]
pub struct IndividualIndex(pub oc_individual::IndividualIndex);

// FIXME: organize (Components also in crates/oc_battle_gui/src/ingame/individual.rs)
#[derive(Debug, Component)]
pub struct Behavior(pub behavior::Behavior);

#[derive(Debug, Component)]
pub struct Orders(pub Vec<oc_individual::order::Order>);
