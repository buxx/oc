use bevy::prelude::*;
use oc_individual::network::Individual;
use oc_network::ToClient;
use oc_projectile::network::Projectile;
use oc_root::Wcfg;

use crate::{
    fx::FxEvent,
    ingame::{
        GameConfigReceived,
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
    states::GameConfig,
    world::InsertTiles,
};

pub fn on_to_client(
    to_client: On<ToClientEvent>,
    mut commands: Commands,
    mut g: ResMut<GameConfig>,
    mut w: ResMut<Wcfg>,
) {
    tracing::trace!(name = "ingame-input-client");

    match &to_client.0 {
        ToClient::GameConfig(config) => {
            tracing::debug!("Received GameConfig");
            g.0 = Some(config.clone());
            w.0 = Some(config.w.clone());
            commands.trigger(GameConfigReceived);
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
        ToClient::Fx(fx) => {
            tracing::trace!(name="ingame-input-fx", fx=?fx);
            commands.trigger(FxEvent(fx.clone()));
        }
    }
}
