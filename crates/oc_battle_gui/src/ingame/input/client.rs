use bevy::prelude::*;
use oc_individual::network;
use oc_network::ToClient;

use crate::{
    ingame::input::individual::{InsertIndividualEvent, UpdateIndividualEvent},
    network::input::ToClientEvent,
    states::{Config, Meta},
};

pub fn on_to_client(
    to_client: On<ToClientEvent>,
    mut commands: Commands,
    mut meta: ResMut<Meta>,
    mut config: ResMut<Config>,
) {
    match &to_client.0 {
        ToClient::Config(config_) => {
            tracing::debug!("Set 'Config'");
            config.0 = Some(config_.clone());
        }
        ToClient::Meta(meta_) => {
            tracing::debug!("Set 'Meta'");
            meta.0 = Some(meta_.clone());
        }
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
