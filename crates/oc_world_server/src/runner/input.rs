use std::sync::{Arc, mpsc::Sender};

use derive_more::Constructor;
use message_io::network::Endpoint;
use oc_geo::region::WorldRegionIndex;
use oc_network::{ToClient, ToServer};
#[cfg(feature = "debug")]
use oc_projectile::network::Projectile;
#[cfg(feature = "debug")]
use oc_projectile::network::SpawnProjectile;
use oc_utils::error::OkOrLogError;
use oc_world::tile::IntoTiles;

use crate::{network::IntoNetworkInsert, state::State, utils::subject::IntoSubject};

#[derive(Constructor)]
pub struct Dealer<'a> {
    state: &'a Arc<State>,
    output: &'a Sender<(Endpoint, ToClient)>,
    endpoint: Endpoint,
}

impl<'a> Dealer<'a> {
    pub fn deal(&self, message: ToServer) {
        match message {
            ToServer::ListenRegion(region) => self.listen_region(region),
            ToServer::ForgotRegion(region) => self.forgot_region(region),
            ToServer::Refresh => self.refresh(),
            #[cfg(feature = "debug")]
            ToServer::SpawnProjectile(projectile) => self.spawn_projectile(projectile),
        }
    }

    fn listen_region(&self, region: WorldRegionIndex) {
        tracing::trace!(name="dealer-listen-region", endpoint=?self.endpoint, region=?region);
        let mut listeners = self.state.listeners_mut();
        listeners.listen_region(&self.endpoint, region);
        self.refresh_region(region);
    }

    fn forgot_region(&self, region: WorldRegionIndex) {
        tracing::trace!(name="dealer-forgot-region", endpoint=?self.endpoint, region=?region);
        let mut listeners = self.state.listeners_mut();
        listeners.forgot_region(&self.endpoint, region);
    }

    fn refresh(&self) {
        // global refresh will send "non region" things (global game info, etc)
    }

    #[cfg(feature = "debug")]
    fn spawn_projectile(&self, projectile: SpawnProjectile) {
        use oc_geo::region::Region;

        use crate::routing::Listening;

        // Insert it in the world
        tracing::trace!(name="dealer-spawn-projectile", endpoint=?self.endpoint, projectile=?projectile);
        let id = self.state.new_projectile_id();
        let mut world = self.state.world_mut();
        let projectiles = world.projectiles_mut();
        projectiles.insert(id, projectile.0.clone());

        // Make it known by index (TODO: normalize ?)
        let mut indexes = self.state.indexes_mut();
        indexes.insert_projectile(id, &projectile.0);

        // Broadcast the new projectile (TODO: normalize ?)
        let region: WorldRegionIndex = projectile.region().clone().into();
        let listeners = self.state.listeners();
        for listener in listeners.find(Listening::Regions(vec![region])) {
            let message = ToClient::Projectile(Projectile::Insert(id, projectile.0.clone()));
            let message = (listener.clone(), message);
            self.output.send(message).ok_or_log();
        }
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
}
