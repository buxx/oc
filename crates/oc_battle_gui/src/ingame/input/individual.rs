use bevy::prelude::*;
use oc_individual::{Individual, IndividualIndex, squad::SquadIndex};
use oc_physics::update::bevy::UpdatePhysicsEvent;
use oc_projectile::ProjectileId;
use oc_root::{
    WcfgFrom,
    geo::{ScreenVec2, WorldVec2},
};
use oc_utils::{let_ok, let_some};

use crate::{ingame::squad::menu::contextual::PrepareOpenSquadContextualMenu, states::GameConfig};

#[derive(Debug, Event)]
pub struct InsertIndividualEvent(pub IndividualIndex, pub Individual);

#[derive(Debug, Event)]
pub struct UpdateIndividualPhysicsEvent(pub IndividualIndex, pub oc_physics::update::Update);

// TODO: move in projectile.rs ?
#[derive(Debug, Event)]
pub struct UpdateProjectilePhysicsEvent(pub ProjectileId, pub oc_physics::update::Update);

// TODO: move in squad.rs
#[derive(Debug, Event)]
pub struct UpdateIndividualEvent(pub IndividualIndex, pub oc_individual::Update);

#[derive(Debug, Event)]
pub struct UpdateSquadEvent(pub SquadIndex, pub oc_individual::squad::Update);

// TODO: derive ?
impl UpdatePhysicsEvent<IndividualIndex> for UpdateIndividualPhysicsEvent {
    fn i(&self) -> IndividualIndex {
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
    camera: Single<(&Camera, &GlobalTransform)>,
) {
    let_some!(g = &g.0, return);
    let (camera, transform) = *camera;
    let_some!(point = click.hit.position, return);
    let point = Vec2::new(point.x, point.y);
    let point = camera.viewport_to_world_2d(transform, point);
    let_ok!(point = point, return);
    let point = ScreenVec2::new(point.x, point.y);
    let point = WorldVec2::from_(point, &g.w);

    if !ingame.selected_squads().is_empty() {
        if click.button == PointerButton::Secondary {
            commands.trigger(PrepareOpenSquadContextualMenu(point));
        }
    }

    click.propagate(false);
}
