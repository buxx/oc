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
use oc_root::{INDIVIDUAL_TICK_INTERVAL_US, INDIVIDUALS_COUNT};
use rayon::{iter::ParallelIterator, slice::ParallelSlice};
use thiserror::Error;

use crate::{individual, network::Event, runner::input::Dealer, state::State};

mod input;

#[derive(Constructor)]
pub struct Runner {
    state: Arc<State>,
    output: Sender<(Endpoint, ToClient)>,
}

impl Runner {
    pub fn run(&self, input: Receiver<Event>) -> Result<(), RunError> {
        self.listen_input(input);
        self.start_individuals();
        self.track_perfs();

        Ok(())
    }

    fn start_individuals(&self) {
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
                            let state = state.clone();
                            let output = output.clone();
                            state.perf.incr();

                            individual::Processor::new((*i).into(), state, output).run();
                        }

                        last = Instant::now();
                    }
                });
            });
    }

    fn track_perfs(&self) {
        loop {
            std::thread::sleep(Duration::from_secs(1));
            println!("{} tick/s", self.state.perf.ticks());
            self.state.perf.reset();
        }
    }

    fn listen_input(&self, input: Receiver<Event>) {
        let state = self.state.clone();
        let output = self.output.clone();

        std::thread::spawn(move || {
            while let Ok(message) = input.recv() {
                match message {
                    Event::Connected(endpoint) => state.listeners_mut().push(endpoint),
                    Event::Disconnected(endpoint) => state.listeners_mut().remove(endpoint),
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
