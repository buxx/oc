use bevy::color::palettes::css::PURPLE;
use bevy::prelude::*;
use oc_geo::region::{RegionXy, WorldRegionIndex};
use oc_geo::tile::TileXy;
use oc_physics::Force;

use crate::entity::geo::Position;
use crate::entity::individual::{Behavior, IndividualIndex};
use crate::entity::physics::Forces;
use crate::entity::world::{Region, Tile};
use crate::ingame::draw::Z_INDIVIDUAL;
use crate::ingame::input::{InsertIndividualEvent, UpdateIndividualEvent};
use crate::ingame::region::ForgottenRegion;
use crate::ingame::state::State;

#[derive(Debug, Event)]
pub struct UpdatePositionEvent(oc_individual::IndividualIndex, [f32; 2]);

#[derive(Debug, Event)]
pub struct UpdateTileEvent(oc_individual::IndividualIndex, TileXy);

#[derive(Debug, Event)]
pub struct UpdateRegionEvent(oc_individual::IndividualIndex, WorldRegionIndex);

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
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    tracing::debug!(
        "Spawn individual {} at {:?}",
        individual.0.0,
        individual.1.position
    );
    let entity = commands
        .spawn((
            IndividualIndex(individual.0),
            Position(individual.1.position),
            Tile(individual.1.tile),
            Region(individual.1.region),
            Behavior(individual.1.behavior),
            Forces(individual.1.forces.clone()),
            Mesh2d(meshes.add(Circle::new(2.5))),
            MeshMaterial2d(materials.add(Color::from(PURPLE))),
            Transform::from_xyz(
                individual.1.position[0],
                individual.1.position[1],
                Z_INDIVIDUAL,
            ),
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
        oc_individual::Update::UpdateTile(tile) => {
            commands.trigger(UpdateTileEvent(i, *tile));
        }
        oc_individual::Update::UpdateRegion(region) => {
            commands.trigger(UpdateRegionEvent(i, *region));
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
            .add_observer(on_update_region_event)
            .add_observer(on_update_behavior_event)
            .add_observer(on_push_force_event)
            .add_observer(on_remove_force_event)
            .add_observer(on_forgotten_region);
    }
}

fn on_update_position_event(
    position: On<UpdatePositionEvent>,
    mut query: Query<(&mut Position, &mut Transform)>,
    state: Res<State>,
) {
    let Some(entity) = state.individuals.get(&position.0) else {
        return;
    };
    let Ok((mut position_, mut transform)) = query.get_mut(*entity) else {
        return;
    };

    tracing::debug!(
        "Update individual {} position for {:?}",
        position.0.0,
        position.1
    );
    position_.0 = position.1;
    transform.translation = Vec3::new(position.1[0], position.1[1], Z_INDIVIDUAL);
}

fn on_update_tile_event(tile: On<UpdateTileEvent>, mut query: Query<&mut Tile>, state: Res<State>) {
    let Some(entity) = state.individuals.get(&tile.0) else {
        return;
    };
    let Ok(mut tile_) = query.get_mut(*entity) else {
        return;
    };

    tracing::debug!("Update individual {} tile for {:?}", tile.0.0, tile.1);
    tile_.0 = tile.1;
}

fn on_update_region_event(
    region: On<UpdateRegionEvent>,
    mut query: Query<&mut Region>,
    state: Res<State>,
) {
    let Some(entity) = state.individuals.get(&region.0) else {
        return;
    };
    let Ok(mut region_) = query.get_mut(*entity) else {
        return;
    };

    tracing::debug!("Update individual {} region for {:?}", region.0.0, region.1);
    region_.0 = region.1;
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

    tracing::debug!(
        "Update individual {} behavior for {:?}",
        behavior.0.0,
        behavior.1
    );
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

    tracing::debug!(
        "Update individual {} pushing force {:?}",
        force.0.0,
        force.1
    );
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

    tracing::debug!(
        "Update individual {} removing force {:?}",
        force.0.0,
        force.1
    );
    forces.0.retain(|f| f != &force.1);
}

fn on_forgotten_region(
    region: On<ForgottenRegion>,
    mut commands: Commands,
    mut state: ResMut<State>,
    query: Query<(Entity, &Region, &IndividualIndex)>,
) {
    for (entity, region_, individual) in query {
        if region_.0 == region.0 {
            commands.entity(entity).despawn();
            state.individuals.remove(&individual.0);
        }
    }
}
