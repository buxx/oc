use std::{
    sync::{
        Arc,
        mpsc::{Receiver, Sender},
    },
    time::{Duration, Instant},
};

use derive_more::Constructor;
use message_io::network::Endpoint;
use oc_network::ToClient;
use oc_root::{INDIVIDUAL_TICK_INTERVAL_US, INDIVIDUALS_COUNT, PHYSICS_TICK_INTERVAL_US};
use rayon::{iter::ParallelIterator, slice::ParallelSlice};
use thiserror::Error;

use crate::{individual, network::Event, physics, runner::input::Dealer, state::State};

mod input;

#[derive(Constructor)]
pub struct Runner {
    state: Arc<State>,
    output: Sender<(Endpoint, ToClient)>,
    print_ticks: bool,
}

impl Runner {
    pub fn run(&self, input: Receiver<Event>) -> Result<(), RunError> {
        self.listen_input(input);
        self.start_physics();
        self.start_individuals();
        self.track_perfs();

        Ok(())
    }

    fn start_physics(&self) {
        let cpus = num_cpus::get();
        let state = self.state.clone();
        let output = self.output.clone();

        (0..cpus).for_each(|i| {
            let state = state.clone();
            let output = output.clone();

            std::thread::spawn(move || {
                let mut last = Instant::now();

                loop {
                    let elapsed = last.elapsed().as_micros() as u64;
                    let wait = PHYSICS_TICK_INTERVAL_US - elapsed;
                    std::thread::sleep(Duration::from_micros(wait));

                    let state = state.clone();
                    let output = output.clone();
                    physics::Processor::new(cpus, state, output).step(i);

                    last = Instant::now();
                }
            });
        });
    }

    fn start_individuals(&self) {
        tracing::debug!("Start individuals threads");

        let cpus = num_cpus::get();
        let state = self.state.clone();
        let output = self.output.clone();
        let size = (INDIVIDUALS_COUNT as f32 / cpus as f32).ceil() as usize;

        (0..INDIVIDUALS_COUNT)
            .collect::<Vec<usize>>()
            .par_chunks(size)
            .for_each(|indexes| {
                let indexes = indexes.to_vec();
                let state = state.clone();
                let output = output.clone();

                std::thread::spawn(move || {
                    let mut last = Instant::now();

                    loop {
                        let elapsed = last.elapsed().as_micros() as u64;
                        let wait = INDIVIDUAL_TICK_INTERVAL_US - elapsed;
                        std::thread::sleep(Duration::from_micros(wait));

                        for i in &indexes {
                            tracing::trace!(name = "runner-individual", i = ?i);

                            let state = state.clone();
                            let output = output.clone();
                            state.perf.incr();

                            individual::Processor::new((*i).into(), state, output).step();
                        }

                        last = Instant::now();
                    }
                });
            });
    }

    fn track_perfs(&self) {
        loop {
            std::thread::sleep(Duration::from_secs(1));

            if self.print_ticks {
                println!("{} tick/s", self.state.perf.ticks());
            }

            self.state.perf.reset();
        }
    }

    fn listen_input(&self, input: Receiver<Event>) {
        let state = self.state.clone();
        let output = self.output.clone();
        let meta = self.state.world().meta().clone();

        std::thread::spawn(move || {
            while let Ok(message) = input.recv() {
                match message {
                    Event::Connected(endpoint) => {
                        state.listeners_mut().push(endpoint.clone());
                        let meta = ToClient::Meta(meta.clone());
                        output.send((endpoint, meta)).unwrap(); // TODO
                    }
                    Event::Disconnected(endpoint) => state.listeners_mut().remove(&endpoint),
                    Event::Message(endpoint, message) => {
                        Dealer::new(&state, &output, endpoint).deal(message);
                    }
                }
            }
        });
    }
}

#[derive(Debug, Error)]
pub enum RunError {}
