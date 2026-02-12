use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use derive_more::Constructor;
use oc_individual::IndividualIndex;
use oc_root::INDIVIDUAL_TICK_INTERVAL_US;

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
        let mut last = Instant::now();

        loop {
            let elapsed = last.elapsed().as_micros() as u64;
            let wait = INDIVIDUAL_TICK_INTERVAL_US - elapsed;
            std::thread::sleep(Duration::from_micros(wait));

            let mut updates = vec![];

            updates.extend(Move::from(&self).read());

            updates.iter().for_each(|update| {
                update::write(update, self.i, &self.state, &self.network);
            });

            self.state.perf.incr();
            last = Instant::now();
        }
    }
}
