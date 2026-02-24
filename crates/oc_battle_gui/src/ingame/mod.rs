use bevy::prelude::*;

#[cfg(feature = "debug")]
use crate::ingame::region::debug;
use crate::{
    ingame::{
        camera::CameraPlugin,
        individual::{IndividualPlugin, on_insert_individual, on_update_individual},
        input::on_to_client,
    },
    states::AppState,
};
use state::State;

mod camera;
mod draw;
mod individual;
mod init;
mod input;
mod region;
mod state;

pub struct IngamePlugin;

impl Plugin for IngamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(IndividualPlugin)
            .add_plugins(CameraPlugin)
            .init_resource::<State>()
            .add_observer(on_to_client)
            .add_observer(on_insert_individual)
            .add_observer(on_update_individual)
            .add_systems(OnEnter(AppState::InGame), init::init);

        #[cfg(feature = "debug")]
        app.add_observer(debug::on_listening_region)
            .add_observer(debug::on_forgotten_region);
    }
}
