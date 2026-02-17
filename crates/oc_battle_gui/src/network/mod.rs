use std::sync::{
    Arc, Mutex,
    mpsc::{Receiver, Sender},
};

use bevy::prelude::*;
use oc_network::{ToClient, ToServer};

pub mod connect;

#[derive(Default)]
pub struct NetworkPlugin;

impl NetworkPlugin {}

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ToServerSender>()
            .init_resource::<ToClientReceiver>();
    }
}

#[derive(Resource, Default)]
pub struct ToServerSender(pub Option<Sender<ToServer>>);

#[derive(Resource, Default)]
pub struct ToClientReceiver(pub Option<Arc<Mutex<Receiver<ToClient>>>>);
