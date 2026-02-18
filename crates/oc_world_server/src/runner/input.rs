use std::sync::{Arc, mpsc::Sender};

use derive_more::Constructor;
use message_io::network::Endpoint;
use oc_geo::tile::TileXy;
use oc_individual::network::Individual;
use oc_network::{ToClient, ToServer};
use oc_utils::error::OkOrLogError;

use crate::{routing::Listen, state::State};

#[derive(Constructor)]
pub struct Dealer<'a> {
    state: &'a Arc<State>,
    output: &'a Sender<(Endpoint, ToClient)>,
    endpoint: Endpoint,
}

impl<'a> Dealer<'a> {
    pub fn deal(&self, message: ToServer) {
        match message {
            ToServer::Listen(from, to) => self.listen(from, to),
            ToServer::Refresh => self.refresh(),
        }
    }

    fn listen(&self, from: TileXy, to: TileXy) {
        tracing::trace!(name="dealer-listen", endpoint=?self.endpoint, from=?from, to=?to);
        self.state
            .listeners_mut()
            .listen(self.endpoint.clone(), Listen::Area(from, to));
    }

    fn refresh(&self) {
        let listeners = self.state.listeners();
        let indexes = self.state.indexes();
        let world = self.state.world();

        let regions = listeners.listener_regions(&self.endpoint);

        for region in regions {
            for i in indexes.region_individuals(*region) {
                let individual = world.individual(*i).clone();
                let message = ToClient::Individual(Individual::Insert(*i, individual.into()));
                let message = (self.endpoint.clone(), message);
                self.output.send(message).ok_or_log();
            }
        }
    }
}
