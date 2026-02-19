use bevy::prelude::*;
use oc_geo::tile::TileXy;
use oc_physics::Force;

use crate::entity::geo::Position;
use crate::entity::individual::{Behavior, IndividualIndex};
use crate::entity::physics::Forces;
use crate::entity::world::Tile;
use crate::ingame::input::{InsertIndividualEvent, UpdateIndividualEvent};
use crate::ingame::state::State;

#[derive(Debug, Event)]
pub struct UpdatePositionEvent(oc_individual::IndividualIndex, [f32; 2]);

#[derive(Debug, Event)]
pub struct UpdateTileEvent(oc_individual::IndividualIndex, TileXy);

#[derive(Debug, Event)]
pub struct UpdateBehaviorEvent(
    oc_individual::IndividualIndex,
    oc_individual::behavior::Behavior,
);

#[derive(Debug, Event)]
pub struct PushForceEvent(oc_individual::IndividualIndex, Force);

#[derive(Debug, Event)]
pub struct RemoveForceEvent(oc_individual::IndividualIndex, Force);

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

pub fn on_update_individual(update: On<UpdateIndividualEvent>, mut commands: Commands) {
    let (i, update) = (update.0, &update.1);

    // TODO: use macro to automatise events declaration and mapping here
    match update {
        oc_individual::Update::UpdatePosition(position) => {
            commands.trigger(UpdatePositionEvent(i, *position));
        }
        oc_individual::Update::UpdateXy(xy) => {
            commands.trigger(UpdateTileEvent(i, *xy));
        }
        oc_individual::Update::UpdateBehavior(behavior) => {
            commands.trigger(UpdateBehaviorEvent(i, *behavior));
        }
        oc_individual::Update::PushForce(force) => {
            commands.trigger(PushForceEvent(i, force.clone()));
        }
        oc_individual::Update::RemoveForce(force) => {
            commands.trigger(RemoveForceEvent(i, force.clone()));
        }
    }
}

pub struct IndividualPlugin;

impl Plugin for IndividualPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_update_position_event)
            .add_observer(on_update_tile_event)
            .add_observer(on_update_behavior_event)
            .add_observer(on_push_force_event)
            .add_observer(on_remove_force_event);
    }
}

fn on_update_position_event(
    position: On<UpdatePositionEvent>,
    mut query: Query<&mut Position>,
    state: Res<State>,
) {
    let Some(entity) = state.individuals.get(&position.0) else {
        return;
    };
    let Ok(mut position_) = query.get_mut(*entity) else {
        return;
    };

    position_.0 = position.1;
}

fn on_update_tile_event(tile: On<UpdateTileEvent>, mut query: Query<&mut Tile>, state: Res<State>) {
    let Some(entity) = state.individuals.get(&tile.0) else {
        return;
    };
    let Ok(mut tile_) = query.get_mut(*entity) else {
        return;
    };

    tile_.0 = tile.1;
}

fn on_update_behavior_event(
    behavior: On<UpdateBehaviorEvent>,
    mut query: Query<&mut Behavior>,
    state: Res<State>,
) {
    let Some(entity) = state.individuals.get(&behavior.0) else {
        return;
    };
    let Ok(mut behavior_) = query.get_mut(*entity) else {
        return;
    };

    behavior_.0 = behavior.1;
}

fn on_push_force_event(
    force: On<PushForceEvent>,
    mut query: Query<&mut Forces>,
    state: Res<State>,
) {
    let Some(entity) = state.individuals.get(&force.0) else {
        return;
    };
    let Ok(mut forces) = query.get_mut(*entity) else {
        return;
    };

    forces.0.push(force.1.clone());
}

fn on_remove_force_event(
    force: On<RemoveForceEvent>,
    mut query: Query<&mut Forces>,
    state: Res<State>,
) {
    let Some(entity) = state.individuals.get(&force.0) else {
        return;
    };
    let Ok(mut forces) = query.get_mut(*entity) else {
        return;
    };

    forces.0.retain(|f| f != &force.1);
}
