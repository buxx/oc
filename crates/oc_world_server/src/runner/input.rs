use std::sync::{Arc, mpsc::Sender};

use derive_more::Constructor;
use message_io::network::Endpoint;
use oc_geo::region::WorldRegionIndex;
use oc_individual::network::Individual;
use oc_network::{ToClient, ToServer};
use oc_utils::error::OkOrLogError;

use crate::state::State;

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
        }
    }

    fn listen_region(&self, region: WorldRegionIndex) {
        tracing::trace!(name="dealer-listen-region", endpoint=?self.endpoint, region=?region);
        self.state
            .listeners_mut()
            .listen_region(&self.endpoint, region);
        self.refresh_region(region);
    }

    fn forgot_region(&self, region: WorldRegionIndex) {
        tracing::trace!(name="dealer-forgot-region", endpoint=?self.endpoint, region=?region);
        self.state
            .listeners_mut()
            .forgot_region(&self.endpoint, region);
    }

    fn refresh(&self) {
        let listeners = self.state.listeners();
        let regions = listeners.listener_regions(&self.endpoint);

        for region in regions {
            self.refresh_region(*region)
        }
    }

    fn refresh_region(&self, region: WorldRegionIndex) {
        tracing::trace!(name="dealer-refresh-region", endpoint=?self.endpoint, region=?region);
        let indexes = self.state.indexes();
        let world = self.state.world();

        for i in indexes.region_individuals(region) {
            let individual = world.individual(*i).clone();
            let message = ToClient::Individual(Individual::Insert(*i, individual.into()));
            let message = (self.endpoint.clone(), message);
            self.output.send(message).ok_or_log();
        }
    }
}
