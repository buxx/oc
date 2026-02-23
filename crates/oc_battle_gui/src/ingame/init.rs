use bevy::prelude::*;
use oc_network::ToServer;

use crate::network::output::ToServerEvent;

pub fn init(mut commands: Commands) {
    commands.trigger(ToServerEvent(ToServer::Refresh.into()));
}
