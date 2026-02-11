use std::{
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};

use derive_more::Constructor;
use oc_root::INDIVIDUAL_TICK_INTERVAL_US;
use oc_world::World;
use thiserror::Error;

use crate::{individual::move_::Move, perf::Perf};

mod effect;
mod move_;

#[derive(Constructor)]
pub struct Processor {
    i: usize,
    perf: Arc<Perf>,
    world: Arc<RwLock<World>>,
}

impl Processor {
    pub fn run(self) -> Result<(), ProcessError> {
        let mut last = Instant::now();

        loop {
            let elapsed = last.elapsed().as_micros() as u64;
            let wait = INDIVIDUAL_TICK_INTERVAL_US - elapsed;
            std::thread::sleep(Duration::from_micros(wait));

            let world = self
                .world
                .read()
                .map_err(|_| ProcessError::PoisonedWorldLock)?;
            let mut effects = vec![];

            effects.extend(Move::from(&self).read(world));

            if !effects.is_empty() {
                let world = self
                    .world
                    .write()
                    .map_err(|_| ProcessError::PoisonedWorldLock)?;
                effect::apply(self.i, world, effects);
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
