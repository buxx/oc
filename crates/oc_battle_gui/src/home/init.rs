use bevy::prelude::*;
use oc_root::{identity::Identity, side::Side};
use uuid::Uuid;

use crate::{config::Config, network::connect::Connect, states::AppState};

pub fn init(
    mut commands: Commands,
    mut args: ResMut<Config>,
    mut state: ResMut<NextState<AppState>>,
) {
    if let Some(connect) = args.0.autoconnect.take() {
        tracing::info!("Auto connect on {connect:?}");
        // TODO: must be accoring to persistent files, etc
        let identity = Identity {
            uuid: Uuid::new_v4(),
            side: Side::A,
        };
        commands.trigger(Connect(connect, identity));
        tracing::debug!("Entering 'Connecting' state");
        *state = NextState::Pending(AppState::Connecting);
    }
}
