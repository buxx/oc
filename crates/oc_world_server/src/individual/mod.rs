use std::{
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};

use derive_more::Constructor;
use oc_root::INDIVIDUAL_TICK_INTERVAL_US;
use oc_world::World;
use thiserror::Error;

use crate::{
    index::Indexes,
    individual::{effect::Apply, move_::Move},
    perf::Perf,
};

mod effect;
mod move_;

#[derive(Constructor)]
pub struct Processor {
    i: usize,
    perf: Arc<Perf>,
    world: Arc<RwLock<World>>,
    indexes: Arc<RwLock<Indexes>>,
}

impl Processor {
    pub fn run(self) -> Result<(), ProcessError> {
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

            self.perf.incr();
            last = Instant::now();
        }
    }
}

#[derive(Debug, Error)]
pub enum ProcessError {
    #[error("Poisoned world lock")]
    PoisonedWorldLock,
}
