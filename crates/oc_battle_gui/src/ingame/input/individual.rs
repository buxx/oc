use bevy::prelude::*;
use oc_individual::{Individual, IndividualIndex};

#[derive(Debug, Event)]
pub struct InsertIndividualEvent(pub IndividualIndex, pub Individual);

#[derive(Debug, Event)]
pub struct UpdateIndividualEvent(pub IndividualIndex, pub oc_individual::Update);
