use std::hash::Hash;

use bevy::prelude::*;
use oc_geo::{region::RegionXy, tile::TileXy};
use oc_utils::bevy::EntityMapping;

#[derive(Debug, Event)]
pub struct SetPositionEvent<I>(I, [f32; 2]);

#[derive(Debug, Event)]
pub struct SetTileEvent<I>(I, TileXy);

#[derive(Debug, Event)]
pub struct SetRegionEvent<I>(I, RegionXy);

#[derive(Debug, Event)]
pub struct PushForceEvent<I>(I, crate::Force);

#[derive(Debug, Event)]
pub struct RemoveForceEvent<I>(I, crate::Force);

#[derive(Debug, Component)]
pub struct Position(pub [f32; 2]);

#[derive(Debug, Component)]
pub struct Tile(pub TileXy);

#[derive(Debug, Component)]
pub struct Region(pub RegionXy);

#[derive(Debug, Component)]
pub struct Forces(pub Vec<crate::Force>);

pub trait UpdatePhysicsEvent<I> {
    fn i(&self) -> I;
    fn value(&self) -> &super::Update;
}

pub fn on_update_physics<I: Send + Sync + 'static, E: Event + UpdatePhysicsEvent<I>>(
    update: On<E>,
    mut commands: Commands,
) {
    let (i, update) = (update.i(), &update.value());

    match update {
        super::Update::SetPosition(position) => {
            commands.trigger(SetPositionEvent(i, position.clone()));
        }
        super::Update::SetTile(tile) => {
            commands.trigger(SetTileEvent(i, tile.clone()));
        }
        super::Update::SetRegion(region) => {
            commands.trigger(SetRegionEvent(i, region.clone()));
        }
        super::Update::PushForce(force) => {
            commands.trigger(PushForceEvent(i, force.clone()));
        }
        super::Update::RemoveForce(force) => {
            commands.trigger(RemoveForceEvent(i, force.clone()));
        }
    }
}

fn on_set_position_event<I: Hash + Eq + Send + Sync + 'static>(
    event: On<SetPositionEvent<I>>,
    mut query: Query<(&mut Position, &mut Transform)>,
    state: Res<EntityMapping<I>>,
) {
    let Some(entity) = state.get(&event.0) else {
        return;
    };
    let Ok((mut position_, mut transform)) = query.get_mut(*entity) else {
        return;
    };
    // tracing::trace!(name = "update-individual-position", i=?position.0.0, position=?position.1);

    position_.0 = event.1;
    transform.translation.x = event.1[0];
    transform.translation.y = event.1[1];
}

fn on_set_tile_event<I: Hash + Eq + Send + Sync + 'static>(
    event: On<SetTileEvent<I>>,
    mut query: Query<&mut Tile>,
    state: Res<EntityMapping<I>>,
) {
    let Some(entity) = state.get(&event.0) else {
        return;
    };
    let Ok(mut component) = query.get_mut(*entity) else {
        return;
    };
    // tracing::trace!(name="update-individual-tile", i=?tile.0, tile=?tile.1);

    component.0 = event.1;
}

fn on_set_region_event<I: Hash + Eq + Send + Sync + 'static>(
    event: On<SetRegionEvent<I>>,
    mut query: Query<&mut Region>,
    state: Res<EntityMapping<I>>,
) {
    let Some(entity) = state.get(&event.0) else {
        return;
    };
    let Ok(mut component) = query.get_mut(*entity) else {
        return;
    };
    // tracing::trace!(name="update-individual-region", i=?region.0, region=?region.1);

    component.0 = event.1;
}

fn on_push_force_event<I: Hash + Eq + Send + Sync + 'static>(
    event: On<PushForceEvent<I>>,
    mut query: Query<&mut Forces>,
    state: Res<EntityMapping<I>>,
) {
    let Some(entity) = state.get(&event.0) else {
        return;
    };
    let Ok(mut component) = query.get_mut(*entity) else {
        return;
    };
    // tracing::trace!(name = "update-individual-force-push", i=?force.0, force=?force.1);

    component.0.push(event.1.clone());
}

fn on_remove_force_event<I: Hash + Eq + Send + Sync + 'static>(
    event: On<RemoveForceEvent<I>>,
    mut query: Query<&mut Forces>,
    state: Res<EntityMapping<I>>,
) {
    let Some(entity) = state.get(&event.0) else {
        return;
    };
    let Ok(mut component) = query.get_mut(*entity) else {
        return;
    };
    // tracing::trace!(name = "update-individual-force-remove", i=?force.0, force=?force.1);

    component.0.retain(|f| f != &event.1);
}

#[derive(Debug)]
pub struct PhysicsPlugin<I: Hash + Eq + Send + Sync + 'static, E: Event + UpdatePhysicsEvent<I>> {
    _marker: std::marker::PhantomData<(I, E)>,
}

impl<I: Hash + Eq + Send + Sync + 'static, E: Event + UpdatePhysicsEvent<I>> Default
    for PhysicsPlugin<I, E>
{
    fn default() -> Self {
        Self {
            _marker: Default::default(),
        }
    }
}

impl<I: Hash + Eq + Send + Sync + 'static, E: Event + UpdatePhysicsEvent<I>> Plugin
    for PhysicsPlugin<I, E>
{
    fn build(&self, app: &mut App) {
        app.add_observer(on_update_physics::<I, E>)
            .add_observer(on_set_position_event::<I>)
            .add_observer(on_set_tile_event::<I>)
            .add_observer(on_set_region_event::<I>)
            .add_observer(on_push_force_event::<I>)
            .add_observer(on_remove_force_event::<I>);
    }
}
