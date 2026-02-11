use std::{
    sync::{Arc, RwLock},
    time::Duration,
};

use derive_more::Constructor;
use oc_root::INDIVIDUALS_COUNT;
use oc_world::World;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use thiserror::Error;

use crate::{individual, perf::Perf};

#[derive(Constructor)]
pub struct Runner {
    world: World,
}

impl Runner {
    pub fn run(self) -> Result<(), RunError> {
        let Self { world } = self;

        let perf = Arc::new(Perf::default());
        let world = Arc::new(RwLock::new(world));

        (0..INDIVIDUALS_COUNT).into_par_iter().for_each(|i| {
            let perf = perf.clone();
            let world = world.clone();
            std::thread::spawn(move || individual::Processor::new(i, perf, world).run());
        });

        loop {
            std::thread::sleep(Duration::from_secs(1));

            println!("{} tick/s", perf.ticks());
            perf.reset();
        }
    }
}

#[derive(Debug, Error)]
pub enum RunError {}
