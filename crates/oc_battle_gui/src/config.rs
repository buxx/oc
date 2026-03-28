use std::{
    net::SocketAddr,
    sync::{
        Arc, Mutex,
        mpsc::{Receiver, Sender},
    },
};

use bevy::prelude::*;
use bon::Builder;
use oc_network::{ToClient, ToServer};

use crate::network::input::NetworkMessage;

#[derive(Debug, Clone, Default, Builder)]
pub struct Config_ {
    pub autoconnect: Option<Connect>,
}

#[derive(Debug, Clone)]
pub enum Connect {
    Network(SocketAddr),
    Embedded(Sender<ToServer>, Arc<Mutex<Receiver<NetworkMessage>>>),
}

#[derive(Resource)]
pub struct Config(pub Config_);
