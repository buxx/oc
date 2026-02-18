use bevy::prelude::*;

use crate::ingame::{
    individual::{on_insert_individual, on_update_individual},
    input::on_to_client,
};
use state::State;

mod individual;
mod input;
mod state;

pub struct IngamePlugin;

impl Plugin for IngamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<State>()
            .add_observer(on_to_client)
            .add_observer(on_insert_individual)
            .add_observer(on_update_individual);
    }
}
