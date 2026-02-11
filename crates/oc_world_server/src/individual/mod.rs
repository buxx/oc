use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use derive_more::Constructor;
use oc_root::INDIVIDUAL_TICK_INTERVAL_US;

use crate::{
    individual::{effect::Apply, move_::Move},
    state::State,
};

mod effect;
mod move_;

#[derive(Constructor)]
pub struct Processor {
    i: usize,
    state: Arc<State>,
}

impl Processor {
    pub fn run(self) {
        let mut last = Instant::now();

        loop {
            let elapsed = last.elapsed().as_micros() as u64;
            let wait = INDIVIDUAL_TICK_INTERVAL_US - elapsed;
            std::thread::sleep(Duration::from_micros(wait));

            let mut effects = vec![];

            effects.extend(Move::from(&self).read());

            if !effects.is_empty() {
                Apply::from(&self).apply(effects);
            }

            self.state.perf.incr();
            last = Instant::now();
        }
    }
}
