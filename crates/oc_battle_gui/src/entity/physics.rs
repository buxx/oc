use bevy::prelude::*;
use oc_physics::Force;

#[derive(Debug, Component)]
pub struct Forces(pub Vec<Force>);
