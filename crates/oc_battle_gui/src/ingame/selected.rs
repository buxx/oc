use bevy::prelude::*;

use crate::ingame::state::State;

#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct SelectedGizmos;

#[derive(Debug, Event, Deref, DerefMut)]
pub struct Select(pub Selected);

#[derive(Debug, Clone, Copy, Component)]
pub enum Selected {
    Individual(oc_individual::IndividualIndex),
}

impl Selected {
    fn is(&self, state: &State) -> bool {
        match self {
            Selected::Individual(i) => state.selected_squads_individuals().contains(i),
        }
    }

    pub fn color(&self) -> Srgba {
        bevy::color::palettes::css::BLUE
    }

    pub fn size(&self) -> Vec2 {
        Vec2::splat(10.)
    }
}

pub fn setup(mut config: ResMut<GizmoConfigStore>) {
    tracing::trace!(name = "utils-selected-setup");
    let (gizmos, _) = config.config_mut::<SelectedGizmos>();
    gizmos.line.width = 1.0;
}

pub fn draw(
    mut gizmos: Gizmos<SelectedGizmos>,
    q: Query<(&Transform, &Selected)>,
    state: Res<State>,
) {
    for (transform, selected) in &q {
        if selected.is(&state) {
            let size = selected.size();
            let isometry = Isometry2d::from_translation(transform.translation.truncate());
            gizmos.rect_2d(isometry, size, selected.color());
        }
    }
}
