use std::{
    net::SocketAddr,
    sync::{Arc, Mutex, mpsc::channel},
};

use bevy::prelude::*;

use crate::network::state::State;
use crate::network::{input::NetworkMessageReceiver, start_network};
use crate::states::AppState;
use crate::{error::OkOrSendError, network::output::ToServerSender};

#[derive(Event)]
pub struct Connect(pub SocketAddr);

#[derive(Event)]
pub struct Connected(pub SocketAddr);

// TODO: manage reconnection
#[derive(Event)]
pub struct Disconnected(pub SocketAddr);

// TODO: display it in gui
#[derive(Event)]
pub struct FailedToConnect(pub SocketAddr);

pub fn on_connect(
    event: On<Connect>,
    mut to_server: ResMut<ToServerSender>,
    mut network_message: ResMut<NetworkMessageReceiver>,
    commands: Commands,
) {
    let host = event.0;
    let (input_tx, input_rx) = channel();
    let (output_tx, output_rx) = channel();

    start_network(host, input_tx, output_rx).ok_or_send(commands);

    to_server.0 = Some(output_tx);
    network_message.0 = Some(Arc::new(Mutex::new(input_rx)));
}

pub fn on_connected(
    connected: On<Connected>,
    mut network_state: ResMut<State>,
    mut app_state: ResMut<NextState<AppState>>,
) {
    tracing::info!("Connected on {}", connected.0);
    network_state.connected = Some(connected.0.clone());
    *app_state = NextState::Pending(AppState::InGame);
}
