use std::{
    net::SocketAddr,
    sync::mpsc::{Receiver, Sender, channel},
    time::Duration,
};

use bevy::prelude::*;
use message_io::network::{NetEvent, Transport};
use message_io::node::{self, NodeEvent};
use oc_geo::tile::TileXy;
use oc_network::{ArchivedToClient, ToClient, ToServer};
use oc_utils::d2::Xy;
use rkyv::{api::low::deserialize, rancor::Error, util::AlignedVec};

use crate::network::{ToClientReceiver, ToServerSender};

#[derive(Event)]
pub struct Connect(pub SocketAddr);

pub fn on_connect(
    event: On<Connect>,
    to_server: ResMut<ToServerSender>,
    to_client: ResMut<ToClientReceiver>,
) -> (Receiver<ToClient>, Sender<ToServer>) {
    let (input_tx, input_rx) = channel();
    let (output_tx, output_rx) = channel();

    let (handler, listener) = node::split::<()>();
    let (server, _) = handler
        .network()
        .connect(Transport::FramedTcp, host)
        .unwrap();

    std::thread::spawn(move || {
        listener.for_each(move |event| match event {
            NodeEvent::Network(event) => match event {
                NetEvent::Connected(_endpoint, _ok) => {
                    tracing::info!("Connected to server ({host})");
                }
                NetEvent::Accepted(_, _) => unreachable!(),
                NetEvent::Message(_, bytes_) => {
                    let mut bytes: AlignedVec = rkyv::util::AlignedVec::with_capacity(bytes_.len());
                    bytes.extend_from_slice(bytes_);
                    let message = rkyv::access::<ArchivedToClient, Error>(&bytes).unwrap(); // TODO
                    let message = deserialize::<ToClient, Error>(message).unwrap(); // TODO
                    tracing::trace!(name="network-received", message = ?message);
                }
                NetEvent::Disconnected(_endpoint) => (),
            },
            NodeEvent::Signal(_signal) => {}
        });
    });

    // loop {
    //     let message = ToServer::Listen(TileXy(Xy(0, 0)), TileXy(Xy(10_000, 10_000)));
    //     let bytes = rkyv::to_bytes::<Error>(&message).unwrap(); // TODO
    //     handler.network().send(server, &bytes);
    //     std::thread::sleep(Duration::from_secs(10));
    // }

    (input_rx, output_tx)
}
