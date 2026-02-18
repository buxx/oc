use bevy::prelude::*;

use crate::entity::geo::Position;
use crate::entity::individual::{Behavior, IndividualIndex};
use crate::entity::physics::Forces;
use crate::entity::world::Tile;
use crate::ingame::input::{InsertIndividualEvent, UpdateIndividualEvent};
use crate::ingame::state::State;

pub fn on_insert_individual(
    individual: On<InsertIndividualEvent>,
    mut commands: Commands,
    mut state: ResMut<State>,
) {
    let entity = commands
        .spawn((
            IndividualIndex(individual.0),
            Position(individual.1.position),
            Tile(individual.1.tile),
            Behavior(individual.1.behavior),
            Forces(individual.1.forces.clone()),
        ))
        .id();
    state.individuals.insert(individual.0, entity);
}

pub fn on_update_individual(
    update: On<UpdateIndividualEvent>,
    mut commands: Commands,
    state: ResMut<State>,
) {
    //
}
