use std::net::SocketAddr;
use std::sync::mpsc::{Receiver, Sender, channel};

use derive_more::Constructor;
use message_io::network::{Endpoint, NetEvent, Transport};
use message_io::node::{self};
use oc_network::ArchivedToClient;
use oc_network::{ArchivedToServer, ToClient, ToServer};
use rkyv::api::low::deserialize;
use rkyv::rancor::Error;

#[derive(Constructor)]
pub struct Network {
    output: Sender<(Endpoint, ToClient)>,
}

impl Network {
    pub fn listen(host: SocketAddr) -> (Self, Receiver<Event>) {
        let (input_tx, input_rx) = channel();
        let (output_tx, output_rx) = channel();

        std::thread::spawn(move || listen(host, input_tx, output_rx));

        (Self { output: output_tx }, input_rx)
    }

    // TODO (broadcast according to who should receive this message; zone looked at, etc.)
    pub fn send(&self, endpoints: Vec<Endpoint>, message: ToClient) {
        for endpoint in endpoints {
            self.output.send((endpoint, message.clone())).unwrap(); // TODO
        }
    }
}

pub fn listen(host: SocketAddr, input: Sender<Event>, output: Receiver<(Endpoint, ToClient)>) {
    tracing::info!("Start listening on {}", host);
    let (handler, listener) = node::split::<()>();

    handler
        .network()
        .listen(Transport::FramedTcp, host)
        .unwrap(); // TODO

    std::thread::spawn(move || {
        listener.for_each(move |event| match event.network() {
            NetEvent::Connected(_, _) => unreachable!(),
            NetEvent::Accepted(endpoint, _listener) => {
                tracing::debug!("Client connected from {}", endpoint.addr());
                input.send(Event::Connected(endpoint)).unwrap(); // TODO
            }
            NetEvent::Message(endpoint, bytes) => {
                let message = rkyv::access::<ArchivedToServer, Error>(&bytes[..]).unwrap(); // TODO
                let message = deserialize::<ToServer, Error>(message).unwrap(); // TODO
                tracing::trace!(name="network-received", endpoint = ?endpoint, message = ?message);
                input.send(Event::Message(endpoint, message)).unwrap(); // TODO
            }
            NetEvent::Disconnected(endpoint) => {
                tracing::debug!("Client {} disconnected", endpoint.addr());
                input.send(Event::Disconnected(endpoint)).unwrap(); // TODO
            }
        });
    });

    while let Ok((endpoint, message)) = output.recv() {
        let bytes = rkyv::to_bytes::<Error>(&message).unwrap(); // TODO
        handler.network().send(endpoint, &bytes);
    }

    tracing::debug!("Exit from listening");
}

pub enum Event {
    Connected(Endpoint),
    Disconnected(Endpoint),
    Message(Endpoint, ToServer),
}
