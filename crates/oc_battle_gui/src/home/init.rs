use bevy::prelude::*;

use crate::{Args, network::connect::Connect, states::AppState};

pub fn init(
    mut commands: Commands,
    mut args: ResMut<Args>,
    mut state: ResMut<NextState<AppState>>,
) {
    if let Some(host) = args.0.autoconnect.take() {
        tracing::info!("Auto connect on {host}");
        commands.trigger(Connect(host));
        *state = NextState::Pending(AppState::Connecting);
    }
}
