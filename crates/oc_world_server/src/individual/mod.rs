use std::sync::Arc;

use derive_more::Constructor;
use oc_individual::IndividualIndex;

use crate::{individual::move_::Move, network::Network, state::State};

mod move_;
mod update;

#[derive(Constructor)]
pub struct Processor {
    i: IndividualIndex,
    state: Arc<State>,
    network: Arc<Network>,
}

impl Processor {
    pub fn run(self) {
        let mut updates = vec![];

        updates.extend(Move::from(&self).read());

        updates.iter().for_each(|update| {
            update::write(update, self.i, &self.state, &self.network);
        });
    }
}
