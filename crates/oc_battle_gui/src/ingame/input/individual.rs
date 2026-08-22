use bevy::prelude::*;
use oc_individual::{Individual, squad::SquadIndex};
use oc_physics::update::bevy::UpdatePhysicsEvent;
use oc_projectile::ProjectileId;
use oc_root::{
    WcfgFrom,
    geo::{ScreenVec2, WorldVec2},
};
use oc_utils::let_some;

use crate::{
    cursor_to,
    entity::individual::IndividualIndex,
    ingame::squad::menu::contextual::{self, PrepareOpenSquadContextualMenu},
    menu::contextual::{ContextMenu, close::CloseContextMenu},
    states::GameConfig,
};

#[derive(Debug, Event)]
pub struct InsertIndividualEvent(pub oc_individual::IndividualIndex, pub Individual);

#[derive(Debug, Event)]
pub struct UpdateIndividualPhysicsEvent(
    pub oc_individual::IndividualIndex,
    pub oc_physics::update::Update,
);

// TODO: move in projectile.rs ?
#[derive(Debug, Event)]
pub struct UpdateProjectilePhysicsEvent(pub ProjectileId, pub oc_physics::update::Update);

// TODO: move in squad.rs
#[derive(Debug, Event)]
pub struct UpdateIndividualEvent(
    pub oc_individual::IndividualIndex,
    pub oc_individual::Update,
);

#[derive(Debug, Event)]
pub struct UpdateSquadEvent(pub SquadIndex, pub oc_individual::squad::Update);

// TODO: derive ?
impl UpdatePhysicsEvent<oc_individual::IndividualIndex> for UpdateIndividualPhysicsEvent {
    fn i(&self) -> oc_individual::IndividualIndex {
        self.0
    }

    fn value(&self) -> &oc_physics::update::Update {
        &self.1
    }
}

// TODO: derive ?
impl UpdatePhysicsEvent<ProjectileId> for UpdateProjectilePhysicsEvent {
    fn i(&self) -> ProjectileId {
        self.0
    }

    fn value(&self) -> &oc_physics::update::Update {
        &self.1
    }
}

pub fn on_click(
    mut click: On<Pointer<Click>>,
    g: Res<GameConfig>,
    mut commands: Commands,
    ingame: ResMut<crate::ingame::state::State>,
    input: ResMut<crate::ingame::input::State>,
    camera: Single<(&Camera, &GlobalTransform)>,
    window: Single<&Window>,
    individuals: Query<&IndividualIndex>,
    menu: Query<&ContextMenu<contextual::Menu>>,
) {
    let_some!(g = &g.0, return);

    // Click on individual already open a contextual menu (see crates/oc_battle_gui/src/ingame/individual.rs)
    if individuals.get(click.original_event_target()).is_ok() {
        return;
    }

    let_some!(point = click.hit.position, return);
    let point = Vec2::new(point.x, point.y);
    let point = cursor_to!(point, camera, &g.w, WorldVec2);

    if !ingame.selected_squads().is_empty() {
        if click.button == PointerButton::Secondary {
            // Close possible menu before open new one
            commands.trigger(CloseContextMenu::<contextual::Menu>::default());

            // If a menu already opened, consider user want to close it
            if !menu.is_empty() {
                return;
            }

            let_some!(cursor = window.cursor_position(), return);
            let position = ScreenVec2::from(cursor);
            let position = WorldVec2::from_(position, &g.w);

            // Prevent open menu after map dragging
            if Some(position) == input.first_right_press {
                tracing::debug!("Trigger open squad contextual menu from outside on {point:?}");
                commands.trigger(PrepareOpenSquadContextualMenu(point));
            }
        }
    }

    click.propagate(false);
}
