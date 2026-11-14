use bevy::prelude::*;
use oc_network::ToServer;
use oc_root::side::Side;

use crate::{
    ingame::{EndResumeEvent, GameConfigReceived},
    network::output::ToServerEvent,
};

/// End the battle as soon as game config is received, then exit. Before ending, kill
/// `kill_squads` squads (side, nth squad of this side).
#[derive(Default, Clone)]
pub struct AutoEndPlugin {
    pub kill_squads: Vec<(Side, u32)>,
}

#[derive(Resource)]
struct KillSquads(Vec<(Side, u32)>);

impl Plugin for AutoEndPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(KillSquads(self.kill_squads.clone()))
            .add_observer(on_game_config)
            .add_observer(on_end_resume);
    }
}

fn on_game_config(_: On<GameConfigReceived>, kill: Res<KillSquads>, mut commands: Commands) {
    for (side, nth) in &kill.0 {
        tracing::info!("Auto end: kill squad {nth} of side {side:?}");
        commands.trigger(ToServerEvent(ToServer::DebugKillSquad(*side, *nth)));
    }

    tracing::info!("Auto end: request battle end");
    commands.trigger(ToServerEvent(ToServer::DebugEndBattle));
}

fn on_end_resume(_: On<EndResumeEvent>, mut exit: MessageWriter<AppExit>) {
    tracing::info!("Auto end: battle ended, exit");
    exit.write(AppExit::Success);
}

/// Parse a `<side>:<nth>` (like `a:0`) squad designation
pub fn parse_kill_squad(value: &str) -> Result<(Side, u32), String> {
    let (side, nth) = value
        .split_once(':')
        .ok_or_else(|| format!("'{value}': expected <side>:<nth> (like a:0)"))?;
    let side = match side {
        "a" | "A" => Side::A,
        "b" | "B" => Side::B,
        _ => return Err(format!("'{side}': side must be 'a' or 'b'")),
    };
    let nth = nth.parse().map_err(|e| format!("'{nth}': {e}"))?;
    Ok((side, nth))
}
