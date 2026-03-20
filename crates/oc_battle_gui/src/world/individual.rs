use bevy::prelude::*;
use oc_geo::UpdateGeo;
use oc_physics::{Physic, UpdatePhysic};

use crate::ingame::input::individual::{InsertIndividualEvent, UpdateIndividualPhysicsEvent};
use crate::ingame::region::ForgottenRegion;

pub fn on_insert_individual(insert: On<InsertIndividualEvent>, mut index: ResMut<super::World>) {
    let i = insert.0;
    let individual = &insert.1;

    index.insert_individual(i, individual.clone());
}

// TODO: should be automatized (macro? derive ?)
pub fn on_forgotten_region(region: On<ForgottenRegion>, mut index: ResMut<super::World>) {
    tracing::trace!(name = "world-individual-forgotten-region", region=?region.0);
    index.remove_individuals(region.0);
}

pub fn on_update_individual_physics(
    update: On<UpdateIndividualPhysicsEvent>,
    mut index: ResMut<super::World>,
) {
    let (i, update) = (update.0, &update.1);
    let Some(mut individual) = index.get_individual(i).cloned() else {
        return;
    };

    // Here, must update index with all used values by oc_physics::step
    match update {
        oc_physics::update::Update::SetTile(tile, _) => {
            let position = *individual.position();
            individual.set_tile(*tile);
            index.remove_individual(i, position);
            index.insert_individual(i, individual);
        }
        oc_physics::update::Update::SetVolume(volume, _) => {
            let position = *individual.position();
            individual.set_volume(volume.clone());
            index.remove_individual(i, position);
            index.insert_individual(i, individual);
        }
        oc_physics::update::Update::SetPosition(_, _)
        | oc_physics::update::Update::SetRegion(_, _)
        | oc_physics::update::Update::PushForce(_)
        | oc_physics::update::Update::RemoveForce(_) => {}
    }
}
