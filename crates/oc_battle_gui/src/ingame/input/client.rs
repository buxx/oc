use bevy::prelude::*;
use oc_individual::network;
use oc_network::ToClient;

use crate::{
    ingame::input::individual::{InsertIndividualEvent, UpdateIndividualEvent},
    network::input::ToClientEvent,
};

pub fn on_to_client(to_client: On<ToClientEvent>, mut commands: Commands) {
    match &to_client.0 {
        ToClient::Individual(message) => match message {
            network::Individual::Insert(i, individual) => {
                commands.trigger(InsertIndividualEvent(*i, individual.clone()));
            }
            network::Individual::Update(i, update) => {
                commands.trigger(UpdateIndividualEvent(*i, update.clone()));
            }
        },
    }
}
