use std::sync::mpsc::Sender;

use bevy::prelude::*;

use crate::error::OkOrLogError;

#[derive(Resource, Default)]
pub struct ToServerSender(pub Option<Sender<oc_network::ToServer>>);

#[derive(Debug, Event)]
pub struct ToServer(pub oc_network::ToServer);

pub fn on_to_server(message: On<ToServer>, sender: Res<ToServerSender>) {
    let Some(sender) = &sender.0 else { return };
    sender.send(message.0.clone()).ok_or_log();
}
