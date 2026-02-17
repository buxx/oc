use std::{
    net::SocketAddr,
    sync::mpsc::{Receiver, Sender},
};

use bevy::prelude::*;
use message_io::{
    network::Endpoint,
    node::{self, NodeEvent, NodeHandler},
};
use message_io::{
    network::{NetEvent, Transport},
    node::NodeListener,
};
use oc_network::{ArchivedToClient, ToClient, ToServer};
use rkyv::{api::low::deserialize, rancor::Error, util::AlignedVec};

use crate::{
    error::OkOrLogError,
    network::{
        connect::{on_connect, on_connected},
        input::{NetworkMessageReceiver, on_network_message},
    },
};
use crate::{
    network::{input::NetworkMessage, output::ToServerSender},
    unwrap_or_log,
};

use state::State;

use crate::network::{input::network_message_router, output::on_to_server};

pub mod connect;
pub mod input;
pub mod output;
pub mod state;

#[derive(Default)]
pub struct NetworkPlugin;

impl NetworkPlugin {}

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<State>()
            .init_resource::<NetworkMessageReceiver>()
            .init_resource::<ToServerSender>()
            .add_observer(on_connect)
            .add_observer(on_connected)
            .add_observer(on_network_message)
            .add_observer(on_to_server)
            .add_systems(Update, network_message_router);
    }
}

pub fn start_network(
    host: SocketAddr,
    input: Sender<NetworkMessage>,
    output: Receiver<ToServer>,
) -> Result<(), String> {
    let (handler, listener) = node::split::<()>();
    let (server, _) = handler
        .network()
        .connect(Transport::FramedTcp, host)
        .map_err(|e| format!("Failed to connect {host}: {e}"))?;

    std::thread::spawn(move || listen(host, listener, input).ok_or_log());
    std::thread::spawn(move || send(handler, server, output).ok_or_log());

    Ok(())
}

fn listen(
    host: SocketAddr,
    listener: NodeListener<()>,
    input: Sender<NetworkMessage>,
) -> Result<(), Box<dyn std::error::Error>> {
    listener.for_each(move |event| match event {
        NodeEvent::Network(event) => match event {
            NetEvent::Connected(_endpoint, ok) => {
                tracing::info!("Connected to server ({host})");
                if ok {
                    input
                        .send(NetworkMessage::Connected(host.clone()))
                        .ok_or_log();
                } else {
                    input
                        .send(NetworkMessage::FailToConnect(host.clone()))
                        .ok_or_log();
                }
            }
            NetEvent::Accepted(_, _) => unreachable!(),
            NetEvent::Message(_, bytes_) => {
                let mut bytes: AlignedVec = rkyv::util::AlignedVec::with_capacity(bytes_.len());
                bytes.extend_from_slice(bytes_);
                let message = unwrap_or_log!(
                    rkyv::access::<ArchivedToClient, Error>(&bytes),
                    "Decode message from server"
                );
                let message = unwrap_or_log!(
                    deserialize::<ToClient, Error>(message),
                    "Deserialize message from server"
                );
                tracing::trace!(name="network-received", message = ?message);
                input.send(NetworkMessage::Message(message)).ok_or_log();
            }
            NetEvent::Disconnected(_endpoint) => {
                tracing::info!("Disconnected from server ({host})");
                input.send(NetworkMessage::Disconnected).ok_or_log();
            }
        },
        NodeEvent::Signal(_signal) => {}
    });

    Ok(())
}

fn send(
    handler: NodeHandler<()>,
    server: Endpoint,
    output: Receiver<ToServer>,
) -> Result<(), Box<dyn std::error::Error>> {
    while let Ok(message) = output.recv() {
        tracing::trace!(name="network-send", message = ?message);
        let bytes = rkyv::to_bytes::<Error>(&message).unwrap(); // TODO
        handler.network().send(server, &bytes);
    }

    Ok(())
}
