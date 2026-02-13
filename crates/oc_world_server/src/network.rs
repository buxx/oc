use std::net::SocketAddr;
use std::sync::mpsc::{Receiver, Sender, channel};

use message_io::network::{Endpoint, NetEvent, Transport};
use message_io::node::{self};
use oc_network::{ArchivedToServer, ToClient, ToServer};
use rkyv::api::low::deserialize;
use rkyv::rancor::Error;
use rkyv::util::AlignedVec;

pub fn listen(host: SocketAddr) -> (Receiver<Event>, Sender<(Endpoint, ToClient)>) {
    let (input_tx, input_rx) = channel();
    let (output_tx, output_rx) = channel();

    std::thread::spawn(move || listen_(host, input_tx, output_rx));

    (input_rx, output_tx)
}

fn listen_(host: SocketAddr, input: Sender<Event>, output: Receiver<(Endpoint, ToClient)>) {
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
            NetEvent::Message(endpoint, bytes_) => {
                let mut bytes: AlignedVec = rkyv::util::AlignedVec::with_capacity(bytes_.len());
                bytes.extend_from_slice(bytes_);
                let message = rkyv::access::<ArchivedToServer, Error>(&bytes[..]).unwrap(); // TODO
                let message = deserialize::<ToServer, Error>(message).unwrap(); // TODO
                tracing::trace!(name="network-message-received", endpoint = ?endpoint, message = ?message);
                input.send(Event::Message(endpoint, message)).unwrap(); // TODO
            }
            NetEvent::Disconnected(endpoint) => {
                tracing::debug!("Client {} disconnected", endpoint.addr());
                input.send(Event::Disconnected(endpoint)).unwrap(); // TODO
            }
        });
    });

    while let Ok((endpoint, message)) = output.recv() {
        tracing::trace!(name="network-message-send", endpoint = ?endpoint, message = ?message);
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
