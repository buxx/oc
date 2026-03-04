use bevy::prelude::*;
use oc_individual::network::Individual;
use oc_network::ToClient;
use oc_projectile::network::Projectile;

use crate::{
    ingame::input::{
        individual::{InsertIndividualEvent, UpdateIndividualEvent},
        projectile::{InsertProjectileEvent, UpdateProjectileEvent},
    },
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
            Individual::Insert(i, individual) => {
                commands.trigger(InsertIndividualEvent(*i, individual.clone()));
            }
            Individual::Update(i, update) => {
                commands.trigger(UpdateIndividualEvent(*i, update.clone()));
            }
        },
        ToClient::Projectile(message) => match message {
            Projectile::Insert(id, projectile) => {
                tracing::trace!(name="ingame-input-client", message=?to_client.0);
                commands.trigger(InsertProjectileEvent(*id, projectile.clone()));
            }
            Projectile::Update(id, update) => {
                tracing::trace!(name="ingame-input-client", message=?to_client.0);
                commands.trigger(UpdateProjectileEvent(*id, update.clone()));
            }
        },
    }
}
