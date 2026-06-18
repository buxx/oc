use bevy::prelude::*;
use bevy_spritesheet_animation::prelude::*;
use oc_geo::region::WorldRegionIndex;
use oc_physics::Physic;
use oc_physics::collision::{Material, Material_};
use oc_physics::update::bevy::{Forces, PhysicsPlugin, Position, Region, Tile, Volumes};
use oc_root::side::Side;
use oc_root::y::Y;
use oc_utils::bevy::EntityMapping;

use crate::entity::individual::{Behavior, IndividualIndex, Intent, Orders};
use crate::ingame;
use crate::ingame::behavior::{DespawnOrder, DespawnOrders, RefreshOrdersEvent};
use crate::ingame::draw::Z_INDIVIDUAL;
use crate::ingame::input::individual::{
    InsertIndividualEvent, UpdateIndividualEvent, UpdateIndividualPhysicsEvent,
};
use crate::ingame::region::ForgottenRegion;
use crate::sprites::IntoAnimation;
use crate::sprites::soldier::{SoldierAnimationInfos, SoldierAnimations};
use crate::states::{AppState, GameConfig};

#[derive(Debug, Deref, Event)]
pub struct ForgotIndividual(pub oc_individual::IndividualIndex);

#[derive(Debug, Deref, Event)]
pub struct RefreshRender(pub oc_individual::IndividualIndex);

#[derive(Debug, Event)]
pub struct SetBehaviorEvent(
    oc_individual::IndividualIndex,
    oc_individual::behavior::Behavior,
);

#[derive(Debug, Event)]
pub struct SetForcesEvent(oc_individual::IndividualIndex, Vec<oc_physics::Force>);

#[derive(Debug, Event)]
pub struct SetOrdersEvent(
    pub oc_individual::IndividualIndex,
    pub Vec<oc_individual::order::Order>,
);

#[derive(Debug, Event)]
pub struct SetStatusEvent(oc_individual::IndividualIndex, oc_individual::Status);

#[derive(Debug, Event)]
pub struct SetGestureEvent(oc_individual::IndividualIndex, oc_individual::Gesture);

#[derive(Debug, Event)]
pub struct SetIntentEvent(
    oc_individual::IndividualIndex,
    oc_individual::behavior::Intent,
);

#[derive(Debug, Event)]
pub struct AccomplishedEvent(oc_individual::IndividualIndex);

#[derive(Debug, Event)]
pub struct MoveStepAccomplishedEvent(oc_individual::IndividualIndex);

#[derive(Debug, Deref, Component)]
pub struct Status(pub oc_individual::Status);

#[derive(Debug, Deref, Component)]
pub struct Gesture(pub oc_individual::Gesture);

pub fn on_insert_individual(
    individual: On<InsertIndividualEvent>,
    mut commands: Commands,
    g: Res<GameConfig>,
    mut state: ResMut<EntityMapping<oc_individual::IndividualIndex>>,
    animations: Res<SoldierAnimations>,
) {
    let Some(g) = &g.0 else { return };
    tracing::trace!(name="spawn-individual", i=?individual.0, position=?individual.1.position);

    let sprite = animations.sprite();
    let gesture = individual.1.gesture.clone();
    let status = individual.1.status;
    let rotation = gesture.rotation();
    let animation = SoldierAnimationInfos::new(Side::A, status, gesture).animation(&animations);
    let position = individual.1.position;

    let entity = commands
        .spawn((
            IndividualIndex(individual.0),
            Position(position),
            Tile(individual.1.tile),
            Region(individual.1.region),
            Behavior(individual.1.behavior.clone()),
            Intent(individual.1.intent.clone()),
            Forces(individual.1.forces.clone()),
            Status(individual.1.status),
            Orders(individual.1.orders.clone()),
            Gesture(individual.1.gesture.clone()),
            Volumes(individual.1.volumes(position, &g.w, &g.mod_).clone()),
            Material_(individual.1.kind()),
            sprite,
            SpritesheetAnimation::new(animation),
            Transform::from_xyz(
                individual.1.position[0],
                individual.1.position[1].to_gui_y(&g.w),
                Z_INDIVIDUAL,
            )
            .with_rotation(rotation),
        ))
        .id();
    state.insert(individual.0, entity);
    commands.trigger(RefreshOrdersEvent(
        individual.0,
        individual.1.orders.clone(),
    ));
}

fn on_refresh_render(
    individual: On<RefreshRender>,
    mut query: Query<
        (&Status, &Gesture, &mut SpritesheetAnimation, &mut Transform),
        With<IndividualIndex>,
    >,
    state: Res<EntityMapping<oc_individual::IndividualIndex>>,
    animations: Res<SoldierAnimations>,
) {
    let Some(entity) = state.get(&individual.0) else {
        return;
    };
    let Ok((status, gesture, mut animation, mut transform)) = query.get_mut(*entity) else {
        return;
    };

    let animation_ = SoldierAnimationInfos::new(Side::A, status.0, gesture.0.clone());
    let animation_ = animation_.animation(&animations);
    let rotation = gesture.rotation();

    animation.switch(animation_);
    transform.rotation = rotation;
}

pub fn on_update_individual(update: On<UpdateIndividualEvent>, mut commands: Commands) {
    let (i, update) = (update.0, &update.1);
    tracing::trace!(name="ingame-individual-update", i=?i, update=?update);

    // TODO: use macro to automatise events declaration and mapping here
    let refresh = match update {
        oc_individual::Update::SetBehavior(behavior) => {
            commands.trigger(SetBehaviorEvent(i, behavior.clone()));
            false
        }
        oc_individual::Update::SetOrders(orders) => {
            commands.trigger(SetOrdersEvent(i, orders.clone()));
            false
        }
        oc_individual::Update::SetForces(forces) => {
            commands.trigger(SetForcesEvent(i, forces.clone()));
            false
        }
        oc_individual::Update::SetStatus(status) => {
            commands.trigger(SetStatusEvent(i, *status));
            true
        }
        oc_individual::Update::SetGesture(gesture) => {
            commands.trigger(SetGestureEvent(i, gesture.clone()));
            true
        }
        oc_individual::Update::SetIntent(intent) => {
            commands.trigger(SetIntentEvent(i, intent.clone()));
            false
        }
        oc_individual::Update::Accomplished => {
            commands.trigger(AccomplishedEvent(i));
            false
        }
        oc_individual::Update::MoveStepAccomplished => {
            commands.trigger(MoveStepAccomplishedEvent(i));
            false
        }
    };

    if refresh {
        commands.trigger(RefreshRender(i));
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
            .add_observer(on_set_intent_event)
            .add_observer(on_set_gesture_event)
            .add_observer(on_set_forces_event)
            .add_observer(on_forgotten_region)
            .add_observer(on_forgot_individual)
            .add_observer(on_set_status_event)
            .add_observer(on_set_orders_event)
            .add_observer(on_accomplished_event)
            .add_observer(on_move_step_accomplished_event)
            .add_observer(on_refresh_render)
            .add_systems(
                Update,
                ingame::physics::physics_step::<oc_individual::IndividualIndex, IndividualIndex>
                    .run_if(in_state(AppState::InGame)),
            );
    }
}

fn on_set_gesture_event(
    gesture: On<SetGestureEvent>,
    mut query: Query<&mut Gesture>,
    state: Res<EntityMapping<oc_individual::IndividualIndex>>,
) {
    let Some(entity) = state.get(&gesture.0) else {
        return;
    };
    let Ok(mut gesture_) = query.get_mut(*entity) else {
        return;
    };
    tracing::trace!(name = "update-individual-gesture", i=?gesture.0, gesture=?gesture.1);

    gesture_.0 = gesture.1.clone();
}

fn on_set_intent_event(
    intent: On<SetIntentEvent>,
    mut query: Query<&mut Intent>,
    state: Res<EntityMapping<oc_individual::IndividualIndex>>,
) {
    let Some(entity) = state.get(&intent.0) else {
        return;
    };
    let Ok(mut intent_) = query.get_mut(*entity) else {
        return;
    };
    tracing::trace!(name = "update-individual-intent", i=?intent.0, intent=?intent.1);

    intent_.0 = intent.1.clone();
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

    behavior_.0 = behavior.1.clone();
}

fn on_move_step_accomplished_event(
    accomplished: On<MoveStepAccomplishedEvent>,
    mut query: Query<&mut Intent>,
    state: Res<EntityMapping<oc_individual::IndividualIndex>>,
) {
    let Some(entity) = state.get(&accomplished.0) else {
        return;
    };
    let Ok(mut intent) = query.get_mut(*entity) else {
        return;
    };
    tracing::trace!(name = "update-individual-move-step-accomplished", i=?accomplished.0);

    match &mut intent.0 {
        oc_individual::behavior::Intent::Idle(_) => {}
        oc_individual::behavior::Intent::MoveTo(_, path) => {
            if !path.is_empty() {
                path.remove(0);
            }
        }
    }
}

fn on_accomplished_event(
    accomplished: On<AccomplishedEvent>,
    mut query: Query<&mut Orders>,
    state: Res<EntityMapping<oc_individual::IndividualIndex>>,
    mut commands: Commands,
) {
    let Some(entity) = state.get(&accomplished.0) else {
        return;
    };
    let Ok(mut orders) = query.get_mut(*entity) else {
        return;
    };
    tracing::trace!(name = "update-individual-accomplished", i=?accomplished.0);

    if !orders.0.is_empty() {
        let order = orders.0.remove(0);
        commands.trigger(DespawnOrder(accomplished.0, order))
    }
}

fn on_set_orders_event(
    orders: On<SetOrdersEvent>,
    mut query: Query<&mut Orders>,
    state: Res<EntityMapping<oc_individual::IndividualIndex>>,
) {
    let Some(entity) = state.get(&orders.0) else {
        return;
    };
    let Ok(mut orders_) = query.get_mut(*entity) else {
        return;
    };
    tracing::trace!(name = "update-individual-set-orders", i=?orders.0, orders=?orders.1);

    orders_.0 = orders.1.clone();
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
    tracing::trace!(name = "update-individual-set-forces", i=?forces.0, forces=?forces.1);

    forces_.0 = forces.1.clone();
}

fn on_set_status_event(
    status: On<SetStatusEvent>,
    mut query: Query<&mut Status>,
    state: Res<EntityMapping<oc_individual::IndividualIndex>>,
) {
    let Some(entity) = state.get(&status.0) else {
        return;
    };
    let Ok(mut status_) = query.get_mut(*entity) else {
        return;
    };
    tracing::trace!(name = "update-individual-set-status", i=?status.0, status=?status.1);

    status_.0 = status.1;
}

// TODO: should be automatized (macro? derive ?)
fn on_forgotten_region(
    region: On<ForgottenRegion>,
    mut commands: Commands,
    mut state: ResMut<EntityMapping<oc_individual::IndividualIndex>>,
    query: Query<(Entity, &Region, &IndividualIndex)>,
) {
    for (entity, region_, individual) in query {
        let region_: WorldRegionIndex = region_.0;
        if region_ == region.0 {
            tracing::trace!(name = "remove-individual", i=?individual);
            commands.entity(entity).despawn();
            state.remove(&individual.0);
        }
    }
}

pub fn on_forgot_individual(
    individual: On<ForgotIndividual>,
    mut commands: Commands,
    mut individuals: ResMut<EntityMapping<oc_individual::IndividualIndex>>,
) {
    if let Some(entity) = individuals.remove(&individual.0) {
        tracing::trace!(name = "remove-individual", i=?individual);
        commands.entity(entity).despawn();
        commands.trigger(DespawnOrders(individual.0));
    }
}
