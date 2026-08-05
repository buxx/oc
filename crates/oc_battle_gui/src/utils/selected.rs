use std::marker::PhantomData;

use bevy::prelude::*;

#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct SelectedGizmos;

// impl Selected {
//     // fn is(&self, state: &State) -> bool {
//     //     match self {
//     //         Selected::Individual(i) => state.selected_squads_individuals().contains(i),
//     //     }
//     // }

//     // pub fn color(&self) -> Srgba {
//     //     bevy::color::palettes::css::BLUE
//     // }

//     // pub fn size(&self) -> Vec2 {
//     //     Vec2::splat(10.)
//     // }
// }

#[derive(Debug, Component, Default, Deref, DerefMut)]
pub struct Selected(pub bool);

pub fn setup<T: Selection + Send + Sync + 'static>(mut config: ResMut<GizmoConfigStore>) {
    tracing::trace!(name = "utils-selected-setup");
    let (gizmos, _) = config.config_mut::<SelectedGizmos>();
    gizmos.line.width = 1.0;
}

pub fn draw<T: Selection + Send + Sync + 'static>(
    mut gizmos: Gizmos<SelectedGizmos>,
    q: Query<(&Transform, &Selected)>, // FIXME: need I (identifier to not select other types like vehicle)
) {
    for (transform, selected) in &q {
        if selected.0 {
            let size = T::size();
            let isometry = Isometry2d::from_translation(transform.translation.truncate());
            gizmos.rect_2d(isometry, size, T::color());
        }
    }
}

/// Un select all. Observer which select must stop propagation to avoid execute this observer.
fn unselect<T: Selection + Send + Sync + 'static>(
    _: On<Pointer<Click>>,
    mut query: Query<&mut Selected>,
) {
    query.iter_mut().for_each(|mut selected| selected.0 = false);
}

#[derive(Debug, Default)]
pub struct SelectedPlugin<T: Selection + Send + Sync + 'static>(PhantomData<T>);

impl<T: Selection + Send + Sync + 'static> Plugin for SelectedPlugin<T> {
    fn build(&self, app: &mut App) {
        app.init_gizmo_group::<SelectedGizmos>()
            .add_systems(Startup, setup::<T>)
            .add_systems(Update, draw::<T>)
            .add_observer(unselect::<T>);
    }
}

pub trait Selection {
    fn size() -> Vec2;
    fn color() -> Srgba;
}
