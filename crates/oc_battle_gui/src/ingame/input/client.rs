use bevy::prelude::*;
use oc_individual::network::Individual;
use oc_network::ToClient;
use oc_projectile::network::Projectile;

use crate::{
    ingame::{
        individual::ForgotIndividual,
        input::{
            individual::{
                InsertIndividualEvent, UpdateIndividualEvent, UpdateIndividualPhysicsEvent,
                UpdateProjectilePhysicsEvent,
            },
            projectile::InsertProjectileEvent,
        },
        projectile::ForgotProjectile,
    },
    network::input::ToClientEvent,
    states::{Config, Meta, Mod},
    world::InsertTiles,
};

pub fn on_to_client(
    to_client: On<ToClientEvent>,
    mut commands: Commands,
    mut mod_: ResMut<Mod>,
    mut meta: ResMut<Meta>,
    mut config: ResMut<Config>,
) {
    tracing::trace!(name="ingame-input-client", message=?to_client.0);

    match &to_client.0 {
        ToClient::Config(config_) => {
            tracing::debug!("Set 'Config'");
            config.0 = Some(config_.clone());
        }
        ToClient::Mod(mod__) => {
            tracing::debug!("Set 'Mod'");
            mod_.0 = Some(mod__.clone());
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
            Individual::Physics(i, update) => {
                commands.trigger(UpdateIndividualPhysicsEvent(*i, update.clone()));
            }
            Individual::Forgot(i) => {
                commands.trigger(ForgotIndividual(*i));
            }
        },
        ToClient::Projectile(message) => match message {
            Projectile::Insert(id, projectile) => {
                tracing::trace!(name="ingame-input-client-projectile-insert", i=?id);
                commands.trigger(InsertProjectileEvent(*id, projectile.clone()));
            }
            Projectile::Physics(id, update) => {
                tracing::trace!(name="ingame-input-client-projectile-physics", i=?id);
                commands.trigger(UpdateProjectilePhysicsEvent(*id, update.clone()));
            }
            Projectile::Forgot(i) => {
                commands.trigger(ForgotProjectile(*i));
            }
        },
        ToClient::Tiles(region, tiles) => {
            tracing::trace!(name="ingame-input-tiles", region=?region);
            commands.trigger(InsertTiles(*region, tiles.clone()));
        }
    }
}
