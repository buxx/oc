use std::{
    net::SocketAddr,
    sync::{Arc, Mutex, mpsc::Receiver},
};

use bevy::prelude::*;
use oc_geo::tile::TileXy;
use oc_network::ToClient;
use oc_utils::d2::Xy;

use crate::network::{connect::Connected, output::ToServer};

#[derive(Debug, Event)]
pub struct NetWorkMessageEvent(pub NetworkMessage);

#[derive(Resource, Default)]
pub struct NetworkMessageReceiver(pub Option<Arc<Mutex<Receiver<NetworkMessage>>>>);

#[derive(Debug)]
pub enum NetworkMessage {
    Connected(SocketAddr),
    FailToConnect(SocketAddr),
    Disconnected, // TODO: manage reconnection
    Message(ToClient),
}

pub fn network_message_router(mut commands: Commands, messages: Res<NetworkMessageReceiver>) {
    let Some(messages) = &messages.0 else {
        return;
    };
    let messages = messages.lock().expect("Assume mutex");
    let mut messages = messages.try_iter();

    while let Some(message) = messages.next() {
        commands.trigger(NetWorkMessageEvent(message))
    }
}

pub fn on_network_message(message: On<NetWorkMessageEvent>, mut commands: Commands) {
    match &message.0 {
        NetworkMessage::FailToConnect(host) => {
            dbg!(host);
        }
        NetworkMessage::Connected(host) => {
            dbg!(host);
            commands.trigger(ToServer(oc_network::ToServer::Listen(
                TileXy(Xy(0, 0)),
                TileXy(Xy(10, 10)),
            )));

            commands.trigger(Connected(host.clone()));
        }
        NetworkMessage::Disconnected => {
            dbg!("disconnected");
        }
        NetworkMessage::Message(message) => {
            dbg!(message);
        }
    }
}
