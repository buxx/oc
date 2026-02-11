use std::{sync::Arc, time::Duration};

use derive_more::Constructor;
use oc_root::INDIVIDUALS_COUNT;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use thiserror::Error;

use crate::{individual, state::State};

#[derive(Constructor)]
pub struct Runner {
    state: State,
}

impl Runner {
    pub fn run(self) -> Result<(), RunError> {
        let Self { state } = self;
        let state = Arc::new(state);

        // Individuals processors
        (0..INDIVIDUALS_COUNT).into_par_iter().for_each(|i| {
            let state = state.clone();
            std::thread::spawn(move || individual::Processor::new(i, state).run());
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
