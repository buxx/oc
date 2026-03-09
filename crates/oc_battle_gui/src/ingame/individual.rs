use bevy::color::palettes::css::PURPLE;
use bevy::prelude::*;
use oc_geo::region::WorldRegionIndex;
use oc_physics::update::bevy::{Forces, PhysicsPlugin, Position, Region, Tile};
use oc_utils::bevy::EntityMapping;

use crate::entity::individual::{Behavior, IndividualIndex};
use crate::ingame::draw::Z_INDIVIDUAL;
use crate::ingame::input::individual::{
    InsertIndividualEvent, UpdateIndividualEvent, UpdateIndividualPhysicsEvent,
};
use crate::ingame::region::ForgottenRegion;

#[derive(Debug, Event)]
pub struct SetBehaviorEvent(
    oc_individual::IndividualIndex,
    oc_individual::behavior::Behavior,
);

#[derive(Debug, Event)]
pub struct SetForcesEvent(oc_individual::IndividualIndex, Vec<oc_physics::Force>);

pub fn on_insert_individual(
    individual: On<InsertIndividualEvent>,
    mut commands: Commands,
    mut state: ResMut<EntityMapping<oc_individual::IndividualIndex>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    tracing::trace!(name="spawn-individual", i=?individual.0, position=?individual.1.position);
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
    state.insert(individual.0, entity);
}

pub fn on_update_individual(update: On<UpdateIndividualEvent>, mut commands: Commands) {
    let (i, update) = (update.0, &update.1);

    // TODO: use macro to automatise events declaration and mapping here
    match update {
        oc_individual::Update::SetBehavior(behavior) => {
            commands.trigger(SetBehaviorEvent(i, *behavior));
        }
        oc_individual::Update::SetForces(forces) => {
            commands.trigger(SetForcesEvent(i, forces.clone()));
        }
    }
}

pub struct IndividualPlugin;

impl Plugin for IndividualPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PhysicsPlugin::<
            oc_individual::IndividualIndex,
            UpdateIndividualPhysicsEvent,
        >::default())
            .init_resource::<EntityMapping<oc_individual::IndividualIndex>>()
            .add_observer(on_insert_individual)
            .add_observer(on_update_individual)
            .add_observer(on_set_behavior_event)
            .add_observer(on_set_forces_event)
            .add_observer(on_forgotten_region);
    }
}

fn on_set_behavior_event(
    behavior: On<SetBehaviorEvent>,
    mut query: Query<&mut Behavior>,
    state: Res<EntityMapping<oc_individual::IndividualIndex>>,
) {
    let Some(entity) = state.get(&behavior.0) else {
        return;
    };
    let Ok(mut behavior_) = query.get_mut(*entity) else {
        return;
    };
    tracing::trace!(name = "update-individual-behavior", i=?behavior.0, behavior=?behavior.1);

    behavior_.0 = behavior.1;
}

fn on_set_forces_event(
    forces: On<SetForcesEvent>,
    mut query: Query<&mut Forces>,
    state: Res<EntityMapping<oc_individual::IndividualIndex>>,
) {
    let Some(entity) = state.get(&forces.0) else {
        return;
    };
    let Ok(mut forces_) = query.get_mut(*entity) else {
        return;
    };
    tracing::trace!(name = "update-individual-set-force", i=?forces.0, forces=?forces.1);

    forces_.0 = forces.1.clone();
}

// TODO: should be automatized (macro? derive ?)
fn on_forgotten_region(
    region: On<ForgottenRegion>,
    mut commands: Commands,
    mut state: ResMut<EntityMapping<oc_individual::IndividualIndex>>,
    query: Query<(Entity, &Region, &IndividualIndex)>,
) {
    for (entity, region_, individual) in query {
        let region_: WorldRegionIndex = region_.0.into();
        if region_ == region.0 {
            tracing::trace!(name = "remove-individual", i=?individual);
            commands.entity(entity).despawn();
            state.remove(&individual.0);
        }
    }
}
