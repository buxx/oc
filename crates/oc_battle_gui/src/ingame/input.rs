use bevy::prelude::*;
use oc_individual::network::Individual;
use oc_network::ToClient;

use crate::ingame::state::State;
use crate::network::input::ToClientEvent;

pub fn on_to_client(to_client: On<ToClientEvent>, state: ResMut<State>) {
    // FIXME BS NOW
    match &to_client.0 {
        ToClient::Individual(message) => match message {
            Individual::Insert(i, individual) => {
                todo!()
            }
            Individual::Update(i, update) => todo!(),
        },
    }
}
