use bevy::prelude::*;
use oc_network::ToServer;

use crate::{
    config::Config, ingame::FirstIngameEnter, network::output::ToServerEvent, states::Game,
};

pub struct TestsPlugin;

impl Plugin for TestsPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_first_ingame_enter)
            .add_systems(Update, tick);
    }
}

fn tick(mut commands: Commands, config: Res<Config>, game: Res<Game>) {
    if let Some(end) = &config.end {
        if let Some(timeout) = end.timeout {
            if game.started.elapsed() > timeout {
                commands.write_message(AppExit::from_code(1)); // TODO: have codes (x = timeout)
            }
        }
    }
}

fn on_first_ingame_enter(
    _: On<FirstIngameEnter>,
    mut commands: Commands,
    mut config: ResMut<Config>,
) {
    // Projectiles
    while let Some(spawn) = config.projectiles.pop() {
        commands.trigger(ToServerEvent(ToServer::SpawnProjectile(spawn)));
    }
}
