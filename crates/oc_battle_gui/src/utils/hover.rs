use std::marker::PhantomData;

use bevy::prelude::*;
use oc_utils::let_ok;

#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct HoveredGizmos;

#[derive(Debug, Component, Default, Deref, DerefMut)]
pub struct Hovered(bool);

fn setup<T: Hover + Send + Sync + 'static>(mut config: ResMut<GizmoConfigStore>) {
    tracing::trace!(name = "utils-hover-setup");
    let (gizmos, _) = config.config_mut::<HoveredGizmos>();
    gizmos.line.width = 1.0;
}

/// Registered on bundle.observe which wanted to be hovered capable
pub fn over(e: On<Pointer<Over>>, mut q: Query<&mut Hovered>) {
    let_ok!(mut h = q.get_mut(e.event_target()), return);
    h.0 = true;
}

/// Registered on bundle.observe which wanted to be hovered capable
pub fn out(e: On<Pointer<Out>>, mut q: Query<&mut Hovered>) {
    let_ok!(mut h = q.get_mut(e.event_target()), return);
    h.0 = false;
}

pub fn draw<T: Hover + Send + Sync + 'static>(
    mut gizmos: Gizmos<HoveredGizmos>,
    q: Query<(&Transform, &Hovered)>, // FIXME: need I (identifier to not select other types like vehicle)
) {
    for (transform, hovered) in &q {
        if hovered.0 {
            let size = T::size();
            let isometry = Isometry2d::from_translation(transform.translation.truncate());
            gizmos.rect_2d(isometry, size, T::color());
        }
    }
}

#[derive(Debug, Default)]
pub struct HoveredPlugin<T: Hover + Send + Sync + 'static>(PhantomData<T>);

impl<T: Hover + Send + Sync + 'static> Plugin for HoveredPlugin<T> {
    fn build(&self, app: &mut App) {
        app.init_gizmo_group::<HoveredGizmos>()
            .add_systems(Startup, setup::<T>)
            .add_systems(Update, draw::<T>);
    }
}

pub trait Hover {
    fn size() -> Vec2;
    fn color() -> Srgba;
}
