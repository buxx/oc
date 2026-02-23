use bevy::prelude::*;
use oc_individual::network;
use oc_individual::{Individual, IndividualIndex};
use oc_network::ToClient;

use crate::network::input::ToClientEvent;

#[derive(Debug, Event)]
pub struct InsertIndividualEvent(pub IndividualIndex, pub Individual);

#[derive(Debug, Event)]
pub struct UpdateIndividualEvent(pub IndividualIndex, pub oc_individual::Update);

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
