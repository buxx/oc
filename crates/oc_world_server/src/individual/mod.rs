use std::sync::{Arc, mpsc::Sender};

use derive_more::Constructor;
use message_io::network::Endpoint;
use oc_individual::IndividualIndex;
use oc_network::ToClient;

use crate::{individual::move_::Move, state::State};

mod move_;
pub mod physics;
pub mod update;

#[derive(Constructor)]
pub struct Processor {
    i: IndividualIndex,
    state: Arc<State>,
    output: Sender<(Endpoint, ToClient)>,
}

impl Processor {
    pub fn step(self) {
        let mut updates = vec![];

        updates.extend(Move::from(&self).read());

        updates.into_iter().for_each(|update| {
            update::write(update, self.i, &self.state, &self.output);
        });
    }
}
