use std::{
    net::SocketAddr,
    sync::{Arc, Mutex, mpsc::Receiver},
};

use bevy::prelude::*;
use oc_network::ToClient;

use crate::network::connect::{Connected, Disconnected, FailedToConnect};

#[derive(Debug, Event)]
pub struct NetWorkMessageEvent(pub NetworkMessage);

#[derive(Debug, Event)]
pub struct ToClientEvent(pub ToClient);

#[derive(Resource, Default)]
pub struct NetworkMessageReceiver(pub Option<Arc<Mutex<Receiver<NetworkMessage>>>>);

#[derive(Debug)]
pub enum NetworkMessage {
    Connected(SocketAddr),
    FailToConnect(SocketAddr),
    Disconnected(SocketAddr),
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
            commands.trigger(FailedToConnect(host.clone()));
        }
        NetworkMessage::Connected(host) => {
            commands.trigger(Connected(host.clone()));
        }
        NetworkMessage::Disconnected(host) => {
            commands.trigger(Disconnected(host.clone()));
        }
        NetworkMessage::Message(message) => {
            commands.trigger(ToClientEvent(message.clone()));
        }
    }
}
