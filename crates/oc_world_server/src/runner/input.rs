use std::sync::{Arc, mpsc::Sender};

use derive_more::Constructor;
use message_io::network::Endpoint;
use oc_geo::region::WorldRegionIndex;
use oc_mod::Mod;
use oc_network::{ToClient, ToServer};
use oc_projectile::spawn::SpawnProjectile;
use oc_utils::error::OkOrLogError;
use oc_world::tile::IntoTiles;

use crate::{
    network::IntoNetworkInsert, runner::update::Update, schedule::Schedule, state::State,
    utils::subject::IntoSubject,
};

#[derive(Constructor)]
pub struct Dealer<'a> {
    state: &'a Arc<State>,
    mod_: &'a Mod,
    output: &'a Sender<(Endpoint, ToClient)>,
    endpoint: Endpoint,
}

impl<'a> Dealer<'a> {
    pub fn deal(&self, message: ToServer) -> Vec<Update> {
        match message {
            ToServer::ListenRegion(region) => self.listen_region(region),
            ToServer::ForgotRegion(region) => self.forgot_region(region),
            ToServer::Refresh => self.refresh(),
            #[cfg(feature = "debug")]
            ToServer::SpawnProjectile(spawn) => self.spawn_projectile(spawn),
        }
    }

    fn listen_region(&self, region: WorldRegionIndex) -> Vec<Update> {
        tracing::trace!(name="dealer-listen-region", endpoint=?self.endpoint, region=?region);
        let mut listeners = self.state.listeners_mut();
        listeners.listen_region(&self.endpoint, region);
        self.refresh_region(region);
        vec![]
    }

    fn forgot_region(&self, region: WorldRegionIndex) -> Vec<Update> {
        tracing::trace!(name="dealer-forgot-region", endpoint=?self.endpoint, region=?region);
        let mut listeners = self.state.listeners_mut();
        listeners.forgot_region(&self.endpoint, region);
        vec![]
    }

    fn refresh(&self) -> Vec<Update> {
        // global refresh will send "non region" things (global game info, etc)
        vec![]
    }

    fn refresh_region(&self, region: WorldRegionIndex) {
        tracing::trace!(name="dealer-refresh-region", endpoint=?self.endpoint, region=?region);
        let indexes = self.state.indexes();

        self.send_tiles(region);
        self.send_subjects(indexes.region_individuals(region));
        self.send_subjects(indexes.region_projectiles(region));
    }

    fn send_subjects<I, T>(&self, subjects: &Vec<I>)
    where
        I: Clone + IntoSubject<T>,
        T: IntoNetworkInsert<I>,
    {
        let world = self.state.world();

        for i in subjects {
            let Some(subject) = i.into_subject(&world) else {
                continue; // TODO: Possible ?
            };
            let subject = subject.into_network_insert(i.clone());
            let message = (self.endpoint.clone(), subject.into());
            self.output.send(message).ok_or_log();
        }
    }

    fn send_tiles(&self, region: WorldRegionIndex) {
        let world = self.state.world();
        let tiles = region.into_tiles(&world);
        let tiles = ToClient::Tiles(region, tiles);
        let message = (self.endpoint.clone(), tiles);
        self.output.send(message).ok_or_log();
    }

    fn spawn_projectile(&self, spawn: SpawnProjectile) -> Vec<Update> {
        spawn
            .schedule(&self.mod_)
            .iter()
            .map(|(instant, fx)| {
                Update::Schedule(
                    instant.clone(),
                    Box::new(Update::SpawnProjectile(spawn.clone(), *fx)),
                )
            })
            .collect()
    }
}
