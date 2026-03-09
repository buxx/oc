use bevy::prelude::*;
use oc_individual::{Individual, IndividualIndex};
use oc_physics::update::bevy::UpdatePhysicsEvent;
use oc_projectile::ProjectileId;

#[derive(Debug, Event)]
pub struct InsertIndividualEvent(pub IndividualIndex, pub Individual);

#[derive(Debug, Event)]
pub struct UpdateIndividualPhysicsEvent(pub IndividualIndex, pub oc_physics::update::Update);

#[derive(Debug, Event)]
pub struct UpdateProjectilePhysicsEvent(pub ProjectileId, pub oc_physics::update::Update);

#[derive(Debug, Event)]
pub struct UpdateIndividualEvent(pub IndividualIndex, pub oc_individual::Update);

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
