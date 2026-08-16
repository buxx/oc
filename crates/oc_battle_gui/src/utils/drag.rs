use std::marker::PhantomData;

use bevy::prelude::*;
use oc_root::{
    WcfgFrom,
    geo::{ScreenVec2, WorldVec2},
    y::V,
};
use oc_utils::{d2::Direction, let_ok, let_some};

use crate::{cursor_to, states::GameConfig, utils::selected};

#[derive(Debug, Resource, Default, Deref, DerefMut)]
pub struct Cursor(pub Option<(WorldVec2, bool)>); // Start dragging position of cursor; bool = entity under cursor was already selected

#[derive(Debug, Component)]
pub struct Dragged<T: Dragging + std::fmt::Debug + Send + Sync + 'static>(PhantomData<T>, bool);

#[derive(Debug, Component, Clone, Copy)]
pub struct Phantom(pub Entity); // bool = was selected

impl<T: Dragging + std::fmt::Debug + Send + Sync + 'static> Default for Dragged<T> {
    fn default() -> Self {
        Self(PhantomData, false)
    }
}

impl<T: Dragging + std::fmt::Debug + Send + Sync + 'static> std::ops::Deref for Dragged<T> {
    type Target = bool;

    fn deref(&self) -> &Self::Target {
        &self.1
    }
}

impl<T: Dragging + std::fmt::Debug + Send + Sync + 'static> std::ops::DerefMut for Dragged<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.1
    }
}

pub enum Visual {
    Offset,
    Direction,
}

#[derive(Debug)]
pub struct DragPlugin<T: Dragging + std::fmt::Debug + Send + Sync + 'static>(PhantomData<T>);

impl<T: Dragging + std::fmt::Debug + Send + Sync + 'static> Default for DragPlugin<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<T: Dragging + std::fmt::Debug + Send + Sync + 'static> Plugin for DragPlugin<T> {
    fn build(&self, app: &mut App) {
        app.init_resource::<Cursor>()
            .add_observer(on_drag_stop::<T>);

        match T::visual() {
            Visual::Offset => app.add_systems(Update, update_positions::<T>),
            Visual::Direction => app.add_systems(Update, update_directions::<T>),
        };
    }
}

pub trait Dragging {
    /// Implementation must spawn bundle (or trigger event which spawn bundle) owning the given marker
    fn spawn(commands: &mut Commands, marker: Phantom);
    /// Implementation assume consequences of drop
    fn drop(commands: &mut Commands, subject: Entity, point: WorldVec2);
    /// Which visual method to apply
    fn visual() -> Visual;
}

/// Observer to attach on spawned bundle owning `Selected` + `Dragged` components.
pub fn on_drag_start<T>(
    mut event: On<Pointer<DragStart>>,
    mut commands: Commands,
    mut selected: Query<(&Dragged<T>, &mut selected::Selected, Entity)>,
    mut cursor: ResMut<Cursor>,
) where
    T: Dragging + std::fmt::Debug + Send + Sync + 'static,
{
    let_some!(point = event.hit.position, return);
    let point = WorldVec2::new(point.x, point.y);
    tracing::trace!(name="utils-drag-on-drag-start", point=?point);

    // Consider dragged entity as selected
    let target = event.event_target();
    let_ok!((_, mut selected_, _) = selected.get_mut(target), return);
    let was_selected = selected_.0;
    selected_.0 = true;

    // Spawn phantom for each selected entities
    for (_, _, entity) in selected.iter().filter(|(_, selected, _)| selected.0) {
        tracing::trace!(name="utils-drag-on-drag-start-spawn", point=?point, entity=?entity);
        T::spawn(&mut commands, Phantom(entity));
    }

    cursor.0 = Some((point, was_selected));
    event.propagate(false);
}

pub fn on_drag_stop<T>(
    mut event: On<Pointer<DragDrop>>,
    mut commands: Commands,
    g: Res<GameConfig>,
    phantoms: Query<(Entity, &Phantom)>,
    mut dragged: Query<&mut selected::Selected, With<Dragged<T>>>,
    mut cursor: ResMut<Cursor>,
    camera: Single<(&Camera, &GlobalTransform)>,
) where
    T: Dragging + std::fmt::Debug + Send + Sync + 'static,
{
    let_some!(g = &g.0, return);
    let_some!(point = event.hit.position, return);
    tracing::trace!(name="utils-drag-on-drag-stop", point=?point);

    let point = Vec2::new(point.x, point.y);
    let point = cursor_to!(point, camera, &g.w, WorldVec2);
    let_some!(cursor_ = cursor.0, return);
    let_ok!(mut dragged = dragged.get_mut(event.dropped), return);
    let point = WorldVec2::new(point.x, point.y);
    dragged.0 = cursor_.1;
    cursor.0 = None;

    for (entity, phantom) in phantoms {
        if let Visual::Offset = T::visual() {
            commands.entity(entity).despawn()
        }

        tracing::trace!(name="utils-drag-on-drag-stop-drop", point=?point, entity=?entity);
        T::drop(&mut commands, phantom.0, point);
    }

    event.propagate(false);
}

fn update_positions<T: Dragging + std::fmt::Debug + Send + Sync + 'static>(
    mut phantoms: Query<(&Phantom, &mut Transform), Without<Dragged<T>>>,
    cursor: Res<Cursor>,
    mut origins: Query<(&mut Dragged<T>, &mut Transform)>,
    window: Single<&Window>,
    camera: Single<(&Camera, &GlobalTransform)>,
) {
    if phantoms.is_empty() {
        return;
    }

    let_some!(point = window.cursor_position(), return);
    let (camera, camera_transform) = *camera;
    let point = camera.viewport_to_world_2d(camera_transform, point);
    let_ok!(point = point, return);

    let_some!((cursor, _) = cursor.0, return);
    let cursor = Vec2::new(cursor.x, cursor.y);
    let offset = (point - cursor).extend(0.);
    let offset: Vec3 = offset.into();

    for (phantom, mut phantom_transform) in phantoms.iter_mut() {
        let_ok!((_, origin) = origins.get_mut(phantom.0), continue);
        phantom_transform.translation = origin.translation + offset;
        tracing::trace!(name="utils-drag-update-position", point=?point, phantom=?phantom, translation=?phantom_transform.translation);
    }
}

fn update_directions<T: Dragging + std::fmt::Debug + Send + Sync + 'static>(
    mut phantoms: Query<(&Phantom, &mut Transform), With<Dragged<T>>>,
    window: Single<&Window>,
    camera: Single<(&Camera, &GlobalTransform)>,
) {
    if phantoms.is_empty() {
        return;
    }

    let_some!(point = window.cursor_position(), return);
    let (camera, camera_transform) = *camera;
    let point = camera.viewport_to_world_2d(camera_transform, point);
    let_ok!(point = point, return);

    let cursor = glam::Vec2::new(point.x, point.y);

    for (phantom, mut phantom_transform) in phantoms.iter_mut() {
        let reference = phantom_transform.translation;
        let reference = glam::Vec2::new(reference.x, reference.y);
        let direction = Direction::from_points2d(reference, cursor);
        phantom_transform.rotation = direction.bquat(V::Server); // Weird, there is mistake in points natures
        tracing::trace!(name="utils-drag-update-rotation", point=?point, phantom=?phantom, rotation=?phantom_transform.rotation);
    }
}
