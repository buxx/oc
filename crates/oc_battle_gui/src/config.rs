use std::{
    net::SocketAddr,
    sync::{
        Arc, Mutex,
        mpsc::{Receiver, Sender},
    },
};

use bevy::prelude::*;
use bon::Builder;
use oc_network::ToServer;
#[cfg(feature = "test")]
use oc_projectile::spawn::SpawnProjectile;
#[cfg(feature = "test")]
use oc_root::end::End;
use oc_root::files::Connection;

use crate::network::input::NetworkMessage;

#[derive(Debug, Clone, Default, Builder)]
pub struct Config_ {
    pub autoconnect: Option<Connect>,
    #[cfg(feature = "test")]
    pub projectiles: Vec<SpawnProjectile>,
    #[cfg(feature = "test")]
    pub end: Option<End>,
}

#[derive(Debug, Clone)]
pub enum Connect {
    Network(SocketAddr),
    Embedded(Sender<ToServer>, Arc<Mutex<Receiver<NetworkMessage>>>),
}

impl From<Connect> for Connection {
    fn from(value: Connect) -> Self {
        match value {
            Connect::Network(addr) => Connection::Network(addr),
            Connect::Embedded(_, _) => Connection::Embedded,
        }
    }
}

#[derive(Resource, Deref, DerefMut)]
pub struct Config(pub Config_);
