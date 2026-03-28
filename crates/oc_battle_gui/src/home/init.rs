use bevy::prelude::*;

use crate::{config::Config, network::connect::Connect, states::AppState};

pub fn init(
    mut commands: Commands,
    mut args: ResMut<Config>,
    mut state: ResMut<NextState<AppState>>,
) {
    if let Some(connect) = args.0.autoconnect.take() {
        tracing::info!("Auto connect on {connect:?}");
        commands.trigger(Connect(connect));
        tracing::debug!("Entering 'Connecting' state");
        *state = NextState::Pending(AppState::Connecting);
    }
}
