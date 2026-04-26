use std::sync::mpsc::Sender;
#[cfg(feature = "debug")]
use std::time::Instant;

#[cfg(feature = "debug")]
use crate::projectile;
use crate::routing::Listening;
use oc_geo::region::{Region, WorldRegionIndex};
use oc_network::ToClient;
#[cfg(feature = "debug")]
use oc_physics::fx;
#[cfg(feature = "debug")]
use oc_projectile::NextProjectileId;
use oc_projectile::ProjectileId;
#[cfg(feature = "debug")]
use oc_projectile::spawn::SpawnProjectile;
use oc_root::Client;
use oc_utils::error::OkOrLogError;

pub enum Update {
    #[cfg(feature = "debug")]
    Schedule(Instant, Box<Update>),
    #[cfg(feature = "debug")]
    SpawnProjectile(SpawnProjectile, bool), // bool == fx
    RemoveProjectile(ProjectileId),
}

impl<E: Client> super::State<E> {
    pub fn update(&self, update: Update, output: &Sender<(E, ToClient)>) {
        for message in match update {
            #[cfg(feature = "debug")]
            Update::Schedule(instant, update) => self.schedule(instant, *update),
            #[cfg(feature = "debug")]
            Update::SpawnProjectile(spawn, fx) => self.spawn_projectile(spawn, fx),
            Update::RemoveProjectile(id) => self.remove_projectile(id),
        } {
            output.send(message).ok_or_log();
        }
    }

    #[cfg(feature = "debug")]
    fn schedule(&self, instant: Instant, update: Update) -> Vec<(E, ToClient)> {
        self.scheduled().push((instant, update));
        vec![]
    }

    #[cfg(feature = "debug")]
    fn spawn_projectile(&self, spawn: SpawnProjectile, fx: bool) -> Vec<(E, ToClient)> {
        use oc_mod::PickSound;

        let id = self._ids.next_projectile_id();

        let projectile = {
            let world = self.world();
            let mod_ = world.mod_();
            projectile::Builder::new(&self.w, mod_, spawn.clone()).build()
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
        let sound = fx.then(|| self._mod().pick_sound((spawn.weapon, spawn.shot)));
        let sound = sound.flatten();
        let position = *projectile.position();

        listeners
            .find(Listening::Regions(vec![region]))
            .iter()
            .map(|listener| {
                let mut messages = vec![];

                let insert = oc_projectile::network::Projectile::Insert(id, projectile.clone());
                let insert = ToClient::Projectile(insert);
                messages.push((listener.clone(), insert));

                if let Some(sound) = sound {
                    let fx = fx::Fx::Audio(fx::Audio::PlayOnce(sound, position));
                    let fx = ToClient::Fx(fx);
                    messages.push((listener.clone(), fx));
                }

                messages
            })
            .flatten()
            .collect()
    }

    fn remove_projectile(&self, id: ProjectileId) -> Vec<(E, ToClient)> {
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
            let region: WorldRegionIndex = projectile.region();
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
