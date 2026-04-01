use std::hash::Hash;

use bevy::prelude::*;
use oc_geo::{region::RegionXy, tile::TileXy};
use oc_root::y::Y as _;
use oc_utils::bevy::EntityMapping;

#[derive(Debug, Event)]
pub struct SetPositionEvent<I>(pub I, pub [f32; 3], pub [f32; 3]); // new, before

#[derive(Debug, Event)]
pub struct SetTileEvent<I>(pub I, pub TileXy, pub TileXy); // new, before

#[derive(Debug, Event)]
pub struct SetRegionEvent<I>(pub I, pub RegionXy, pub RegionXy); // new, before

#[derive(Debug, Event)]
pub struct PushForceEvent<I>(pub I, pub crate::Force);

#[derive(Debug, Event)]
pub struct RemoveForceEvent<I>(pub I, pub crate::Force);

#[derive(Debug, Event)]
pub struct SetVolumeEvent<I>(pub I, pub crate::volume::Volume, pub crate::volume::Volume); // new, before

#[derive(Debug, Component)]
pub struct Position(pub [f32; 3]);

#[derive(Debug, Component)]
pub struct Tile(pub TileXy);

#[derive(Debug, Component)]
pub struct Region(pub RegionXy);

#[derive(Debug, Component)]
pub struct Forces(pub Vec<crate::Force>);

#[derive(Debug, Component)]
pub struct Volume(pub crate::volume::Volume);

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
        super::Update::SetPosition(position2, position1) => {
            commands.trigger(SetPositionEvent(i, position2.clone(), position1.clone()));
        }
        super::Update::SetTile(tile2, tile1) => {
            commands.trigger(SetTileEvent(i, tile2.clone(), tile1.clone()));
        }
        super::Update::SetRegion(region2, region1) => {
            commands.trigger(SetRegionEvent(i, region2.clone(), region1.clone()));
        }
        super::Update::PushForce(force) => {
            commands.trigger(PushForceEvent(i, force.clone()));
        }
        super::Update::RemoveForce(force) => {
            commands.trigger(RemoveForceEvent(i, force.clone()));
        }
        super::Update::SetVolume(volume2, volume1) => {
            commands.trigger(SetVolumeEvent(i, volume2.clone(), volume1.clone()));
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
    let translation = event.1.to_gui_y();
    transform.translation.x = translation[0];
    transform.translation.y = translation[1];
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

fn on_set_volume<I: Hash + Eq + Send + Sync + 'static>(
    event: On<SetVolumeEvent<I>>,
    mut query: Query<&mut Volume>,
    state: Res<EntityMapping<I>>,
) {
    let Some(entity) = state.get(&event.0) else {
        return;
    };
    let Ok(mut component) = query.get_mut(*entity) else {
        return;
    };
    // tracing::trace!(name="update-individual-region", i=?region.0, region=?region.1);

    component.0 = event.1.clone();
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
            .add_observer(on_set_volume::<I>)
            .add_observer(on_remove_force_event::<I>);
    }
}
