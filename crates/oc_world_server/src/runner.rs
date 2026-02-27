use std::{sync::Arc, time::Duration};

use derive_more::Constructor;
use oc_root::INDIVIDUALS_COUNT;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use thiserror::Error;

use crate::{individual, network::Network, state::State};

#[derive(Constructor)]
pub struct Runner {
    state: State,
    network: Network,
}

impl Runner {
    pub fn run(self) -> Result<(), RunError> {
        let Self { state, network } = self;
        let state = Arc::new(state);
        let network = Arc::new(network);

        // Individuals processors
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(12)
            .build()
            .unwrap();
        pool.install(|| {
            (0..INDIVIDUALS_COUNT).into_par_iter().for_each(|i| {
                let state = state.clone();
                let network = network.clone();
                std::thread::spawn(move || {
                    individual::Processor::new(i.into(), state, network).run()
                });
            })
        });

        // Perf display
        loop {
            std::thread::sleep(Duration::from_secs(1));
            println!("{} tick/s", state.perf.ticks());
            state.perf.reset();
        }
    }
}

#[derive(Debug, Error)]
pub enum RunError {}
