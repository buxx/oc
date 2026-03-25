use std::sync::mpsc::Sender;

use message_io::network::Endpoint;
use oc_network::ToClient;
use oc_projectile::ProjectileId;
#[cfg(feature = "debug")]
use oc_projectile::spawn::SpawnProjectile;
use oc_utils::error::OkOrLogError;

pub enum Update {
    #[cfg(feature = "debug")]
    SpawnProjectile(SpawnProjectile),
    RemoveProjectile(ProjectileId),
}

impl super::State {
    pub fn update(&self, update: Update, output: &Sender<(Endpoint, ToClient)>) {
        for message in match update {
            #[cfg(feature = "debug")]
            Update::SpawnProjectile(spawn) => self.spawn_projectile(spawn),
            // FIXME BS NOW: code it
            Update::RemoveProjectile(id) => vec![],
        } {
            output.send(message).ok_or_log();
        }
    }

    #[cfg(feature = "debug")]
    fn spawn_projectile(&self, spawn: SpawnProjectile) -> Vec<(Endpoint, ToClient)> {
        use crate::projectile;
        use crate::routing::Listening;
        use oc_geo::region::{Region, WorldRegionIndex};

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
}
