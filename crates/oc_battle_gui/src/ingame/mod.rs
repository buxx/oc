use bevy::prelude::*;

use crate::ingame::input::on_to_client;
use state::State;

mod input;
mod state;

pub struct IngamePlugin;

impl Plugin for IngamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<State>().add_observer(on_to_client);
    }
}
