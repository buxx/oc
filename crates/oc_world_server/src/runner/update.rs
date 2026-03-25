use std::{sync::mpsc::Sender, time::Instant};

use crate::projectile;
use crate::routing::Listening;
use message_io::network::Endpoint;
use oc_geo::region::{Region, WorldRegionIndex};
use oc_network::ToClient;
use oc_projectile::ProjectileId;
#[cfg(feature = "debug")]
use oc_projectile::spawn::SpawnProjectile;
use oc_utils::error::OkOrLogError;

pub enum Update {
    Schedule(Instant, Box<Update>),
    #[cfg(feature = "debug")]
    SpawnProjectile(SpawnProjectile),
    RemoveProjectile(ProjectileId),
}

impl super::State {
    pub fn update(&self, update: Update, output: &Sender<(Endpoint, ToClient)>) {
        for message in match update {
            #[cfg(feature = "debug")]
            Update::Schedule(instant, update) => self.schedule(instant, *update),
            Update::SpawnProjectile(spawn) => self.spawn_projectile(spawn),
            Update::RemoveProjectile(id) => self.remove_projectile(id),
        } {
            output.send(message).ok_or_log();
        }
    }

    fn schedule(&self, instant: Instant, update: Update) -> Vec<(Endpoint, ToClient)> {
        self.scheduled().push((instant, update));
        vec![]
    }

    #[cfg(feature = "debug")]
    fn spawn_projectile(&self, spawn: SpawnProjectile) -> Vec<(Endpoint, ToClient)> {
        let id = self.new_projectile_id();

        let projectile = {
            let world = self.world();
            let mod_ = world.mod_();
            projectile::Builder::new(mod_, spawn).build()
        };

        // Make both insert and update index at same clock to lock at the same time
        {
            let mut world = self.world_mut();
            let mut indexes = self.indexes_mut();
            let projectiles = world.projectiles_mut();

            projectiles.insert(id, projectile.clone());
            indexes.insert_projectile(id, &projectile);
        }

        // Broadcast the new projectile (TODO: normalize/refactor to not call loop manually ?)
        let region: WorldRegionIndex = projectile.region().clone().into();
        let listeners = self.listeners();
        listeners
            .find(Listening::Regions(vec![region]))
            .iter()
            .map(|listener| {
                let insert = oc_projectile::network::Projectile::Insert(id, projectile.clone());
                let message = ToClient::Projectile(insert);
                (listener.clone(), message)
            })
            .collect()
    }

    fn remove_projectile(&self, id: ProjectileId) -> Vec<(Endpoint, ToClient)> {
        let projectile = {
            let mut world = self.world_mut();
            let mut indexes = self.indexes_mut();

            if let Some(projectile) = world.projectiles_mut().remove(&id) {
                indexes.remove_projectile(&id, &projectile);
                Some(projectile)
            } else {
                None
            }
        };

        if let Some(projectile) = projectile {
            let listeners = self.listeners();

            // Broadcast the new projectile (TODO: normalize/refactor to not call loop manually ?)
            let region: WorldRegionIndex = projectile.region().clone().into();
            listeners
                .find(Listening::Regions(vec![region]))
                .iter()
                .map(|listener| {
                    let insert = oc_projectile::network::Projectile::Forgot(id);
                    let message = ToClient::Projectile(insert);
                    (listener.clone(), message)
                })
                .collect()
        } else {
            vec![]
        }
    }
}
