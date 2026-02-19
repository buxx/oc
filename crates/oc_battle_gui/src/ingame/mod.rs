use bevy::prelude::*;

use crate::{
    ingame::{
        individual::{IndividualPlugin, on_insert_individual, on_update_individual},
        input::on_to_client,
    },
    states::AppState,
};
use state::State;

mod draw;
mod individual;
mod init;
mod input;
mod state;

pub struct IngamePlugin;

impl Plugin for IngamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(IndividualPlugin)
            .init_resource::<State>()
            .add_observer(on_to_client)
            .add_observer(on_insert_individual)
            .add_observer(on_update_individual)
            .add_systems(OnEnter(AppState::InGame), init::init);
    }
}
