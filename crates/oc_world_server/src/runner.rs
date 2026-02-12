use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use derive_more::Constructor;
use oc_root::{INDIVIDUAL_TICK_INTERVAL_US, INDIVIDUALS_COUNT};
use rayon::{iter::ParallelIterator, slice::ParallelSlice};
use thiserror::Error;

use crate::{individual, network::Network, state::State};

#[derive(Constructor)]
pub struct Runner {
    state: Arc<State>,
    network: Arc<Network>,
}

impl Runner {
    pub fn run(&self) -> Result<(), RunError> {
        self.start_individuals();
        self.track_perfs();
        Ok(())
    }

    fn track_perfs(&self) {
        loop {
            std::thread::sleep(Duration::from_secs(1));
            println!("{} tick/s", self.state.perf.ticks());
            self.state.perf.reset();
        }
    }

    fn start_individuals(&self) {
        let cpus = num_cpus::get();
        let state = self.state.clone();
        let network = self.network.clone();
        let size = (INDIVIDUALS_COUNT as f32 / cpus as f32).ceil() as usize;

        (0..INDIVIDUALS_COUNT)
            .collect::<Vec<usize>>()
            .par_chunks(size)
            .for_each(|indexes| {
                let indexes = indexes.to_vec();
                let state = state.clone();
                let network = network.clone();

                std::thread::spawn(move || {
                    let mut last = Instant::now();

                    loop {
                        let elapsed = last.elapsed().as_micros() as u64;
                        let wait = INDIVIDUAL_TICK_INTERVAL_US - elapsed;
                        std::thread::sleep(Duration::from_micros(wait));

                        for i in &indexes {
                            let state = state.clone();
                            let network = network.clone();
                            state.perf.incr();

                            individual::Processor::new((*i).into(), state, network).run();
                        }

                        last = Instant::now();
                    }
                });
            });
    }
}

#[derive(Debug, Error)]
pub enum RunError {}
