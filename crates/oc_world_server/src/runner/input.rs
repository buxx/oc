use std::sync::{Arc, mpsc::Sender};

use derive_more::Constructor;
use message_io::network::Endpoint;
use oc_geo::tile::TileXy;
use oc_network::{ToClient, ToServer};

use crate::{routing::Listen, state::State};

#[derive(Constructor)]
pub struct Dealer<'a> {
    state: &'a Arc<State>,
    _output: &'a Sender<(Endpoint, ToClient)>,
    endpoint: Endpoint,
}

impl<'a> Dealer<'a> {
    pub fn deal(&self, message: ToServer) {
        match message {
            ToServer::Listen(from, to) => self.listen(from, to),
            ToServer::Refresh => todo!(),
        }
    }

    fn listen(&self, from: TileXy, to: TileXy) {
        tracing::trace!(name="dealer-listen", endpoint=?self.endpoint, from=?from, to=?to);
        self.state
            .listeners_mut()
            .listen(self.endpoint.clone(), Listen::Area(from, to));
    }
}
