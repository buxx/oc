use std::time::Instant;

use crate::projectile;
use crate::routing::Listening;
use crate::utils::context::Context;
use oc_geo::region::{Region, WorldRegionIndex};
use oc_individual::IndividualIndex;
use oc_individual::squad::SquadIndex;
use oc_mod::PickSound;
use oc_network::ToClient;
use oc_physics::fx;
use oc_projectile::NextProjectileId;
use oc_projectile::ProjectileId;
use oc_projectile::spawn::SpawnProjectile;
use oc_root::Client;

#[derive(Debug)]
pub enum Update {
    Schedule(Instant, Box<Update>),
    SpawnProjectile(SpawnProjectile, bool), // bool == fx
    RemoveProjectile(ProjectileId),
    UpdateIndividual(IndividualIndex, oc_individual::Update),
    UpdateSquad(SquadIndex, oc_individual::squad::Update),
}

pub fn update<E: Client>(ctx: &Context<E>, update: Update) {
    let state = &ctx.state;

    for (filter, messages) in match update {
        Update::Schedule(instant, update) => state.schedule(instant, *update),
        Update::SpawnProjectile(spawn, fx) => state.spawn_projectile(spawn, fx),
        Update::RemoveProjectile(i) => state.remove_projectile(i),
        Update::UpdateIndividual(i, update) => {
            #[cfg(feature = "tracker")]
            {
                let mut tracker = ctx.tracker.take();
                tracker.individuals.push((i, update.clone()));
            }
            state.update_individual(i, update)
        }
        Update::UpdateSquad(i, update) => state.update_squad(i, update),
    } {
        tracing::trace!(name="runner-update-broadcast", filter=?filter, messages=?messages);
        ctx.broadcast(filter, messages);
    }
}

impl<E: Client> super::State<E> {
    fn schedule(&self, instant: Instant, update: Update) -> Vec<(Listening, Vec<ToClient>)> {
        self.scheduled().push((instant, update));
        vec![]
    }

    fn update_individual(
        &self,
        i: IndividualIndex,
        update: oc_individual::Update,
    ) -> Vec<(Listening, Vec<ToClient>)> {
        let mut world = self.world_mut();
        crate::individual::update::write(&mut world, update, i)
    }

    fn update_squad(
        &self,
        i: SquadIndex,
        update: oc_individual::squad::Update,
    ) -> Vec<(Listening, Vec<ToClient>)> {
        println!("RUNNER::SQUAD::WRITE::take");
        let mut world = self.world_mut();

        let x = match update {
            oc_individual::squad::Update::Accomplished => {
                let squad = world.squad_mut(i);
                squad.orders.pop();
                vec![]
            }
        };
        println!("RUNNER::SQUAD::WRITE::release");
        x
    }

    fn spawn_projectile(
        &self,
        spawn: SpawnProjectile,
        fx: bool,
    ) -> Vec<(Listening, Vec<ToClient>)> {
        let i = self._ids.next_projectile_id();

        let projectile = {
            let world = self.world();
            let mod_ = world.mod_();
            projectile::Builder::new(&self.w, mod_, spawn.clone()).build()
        };

        // Make both insert and update index at same clock to lock at the same time
        {
            println!("RUNNER::PROJ::WRITE::take");
            let mut world = self.world_mut();
            let mut indexes = self.indexes_mut();
            let projectiles = world.projectiles_mut();

            projectiles.insert(i, projectile.clone());
            indexes.insert_projectile(i, &projectile);
            println!("RUNNER::PROJ::WRITE::release");
        }

        // Broadcast the new projectile (TODO: normalize/refactor to not call loop manually ?)
        let region: WorldRegionIndex = projectile.region();
        let sound = fx.then(|| self._mod().pick_sound((spawn.weapon, spawn.shot)));
        let sound = sound.flatten();
        let position = *projectile.position();

        let mut messages = vec![];

        let insert = oc_projectile::network::Projectile::Insert(i, projectile.clone());
        let insert = ToClient::Projectile(insert);
        messages.push(insert);

        if let Some(sound) = sound {
            let fx = fx::Fx::Audio(fx::Audio::PlayOnce(sound, position));
            let fx = ToClient::Fx(fx);
            messages.push(fx);
        }

        vec![(Listening::Regions(vec![region]), messages)]
    }

    fn remove_projectile(&self, id: ProjectileId) -> Vec<(Listening, Vec<ToClient>)> {
        let projectile = {
            println!("RUNNER::PROJ2::WRITE::take");
            let mut world = self.world_mut();
            let mut indexes = self.indexes_mut();

            let x = if let Some(projectile) = world.projectiles_mut().remove(&id) {
                indexes.remove_projectile(&id, &projectile);
                Some(projectile)
            } else {
                None
            };

            println!("RUNNER::PROJ2::WRITE::take");
            x
        };

        if let Some(projectile) = projectile {
            let region: WorldRegionIndex = projectile.region();
            let insert = oc_projectile::network::Projectile::Forgot(id);
            let message = ToClient::Projectile(insert);
            vec![(Listening::Regions(vec![region]), vec![message])]
        } else {
            vec![]
        }
    }
}
