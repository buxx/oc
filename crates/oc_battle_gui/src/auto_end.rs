use bevy::prelude::*;
use oc_network::ToServer;

use crate::{
    ingame::{EndResumeEvent, GameConfigReceived},
    network::output::ToServerEvent,
};

pub struct AutoEndPlugin;

impl Plugin for AutoEndPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_game_config).add_observer(on_end_resume);
    }
}

fn on_game_config(_: On<GameConfigReceived>, mut commands: Commands) {
    tracing::info!("Auto end: request battle end");
    commands.trigger(ToServerEvent(ToServer::DebugEndBattle));
}

fn on_end_resume(_: On<EndResumeEvent>, mut exit: MessageWriter<AppExit>) {
    tracing::info!("Auto end: battle ended, exit");
    exit.write(AppExit::Success);
}
