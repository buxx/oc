use std::{
    sync::{
        Arc,
        mpsc::{Receiver, Sender},
    },
    time::{Duration, Instant},
};

use derive_more::Constructor;
use oc_network::ToClient;
use oc_root::{Client, INDIVIDUAL_TICK_INTERVAL_US, PHYSICS_TICK_INTERVAL_US};
use rayon::{iter::ParallelIterator, slice::ParallelSlice};

use crate::{
    bridge::Event, config::ServerConfig, individual, physics, runner::input::Dealer, state::State,
    utils::context::Context,
};

pub mod input;
pub mod update;

#[derive(Constructor)]
pub struct Runner<E: Client> {
    config: ServerConfig,
    state: Arc<State<E>>,
    output: Sender<(E, ToClient)>,
}

impl<E: Client> Runner<E> {
    pub fn run(&self, input: Receiver<Event<E>>, ready: Sender<Result<(), String>>) {
        self.listen_input(input);
        self.start_physics();
        self.start_individuals();
        self.start_scheduler();

        let _ = ready.send(Ok(()));
        self.track_perfs();

        tracing::debug!("Finished runner");
    }

    fn start_physics(&self) {
        tracing::debug!("Start physics");
        let ctx = Context::new(self.state.clone(), self.output.clone());

        (0..ctx.cpus).for_each(|i| {
            let ctx = ctx.clone();

            std::thread::spawn(move || {
                let mut last = Instant::now();

                loop {
                    let elapsed = last.elapsed().as_micros() as u64;
                    let wait = PHYSICS_TICK_INTERVAL_US - elapsed;
                    std::thread::sleep(Duration::from_micros(wait));

                    for update in physics::Processor::new(&ctx).step(i) {
                        ctx.state.update(update, &ctx.output);
                    }

                    last = Instant::now();
                }
            });
        });
    }

    fn start_individuals(&self) {
        tracing::debug!("Start individuals");

        let ctx = Context::new(self.state.clone(), self.output.clone());
        let individuals_count = {
            let world = self.state.world();
            world.individuals().len()
        };
        let size = (individuals_count as f32 / ctx.cpus as f32).ceil() as usize;
        if size == 0 {
            return;
        }

        (0..individuals_count)
            .collect::<Vec<usize>>()
            .par_chunks(size)
            .for_each(|indexes| {
                let indexes = indexes.to_vec();
                let ctx = ctx.clone();

                std::thread::spawn(move || {
                    let mut last = Instant::now();

                    loop {
                        let elapsed = last.elapsed().as_micros() as u64;
                        let wait = INDIVIDUAL_TICK_INTERVAL_US - elapsed;
                        std::thread::sleep(Duration::from_micros(wait));

                        for i in &indexes {
                            tracing::trace!(name = "runner-individual", i = ?i);

                            ctx.state.perf.incr();
                            individual::Processor::new(&ctx, (*i).into()).step();
                        }

                        last = Instant::now();
                    }
                });
            });
    }

    fn track_perfs(&self) {
        tracing::debug!("Track perfs");
        loop {
            std::thread::sleep(Duration::from_secs(1));

            if self.config.print_ticks {
                println!("{} tick/s", self.state.perf.ticks());
            }

            self.state.perf.reset();
        }
    }

    fn listen_input(&self, input: Receiver<Event<E>>) {
        tracing::debug!("Listen inputs");
        let state = self.state.clone();
        let output = self.output.clone();
        let mod_ = self.state.world().mod_().clone();
        let meta = self.state.world().meta().clone();
        let static_ = self.config.static_.clone();

        std::thread::spawn(move || {
            while let Ok(message) = input.recv() {
                match message {
                    Event::Connected(endpoint) => {
                        state.listeners_mut().push(endpoint.clone());
                        let mod_ = ToClient::Mod(mod_.clone());
                        output.send((endpoint.clone(), mod_)).unwrap(); // TODO
                        let meta = ToClient::Meta(meta.clone());
                        output.send((endpoint.clone(), meta)).unwrap(); // TODO
                        let config = ToClient::StaticSource(static_.clone());
                        output.send((endpoint.clone(), config)).unwrap(); // TODO
                    }
                    Event::Disconnected(endpoint) => state.listeners_mut().remove(&endpoint),
                    Event::Message(endpoint, message) => {
                        for update in Dealer::new(&state, &mod_, &output, endpoint).deal(message) {
                            state.update(update, &output);
                        }
                    }
                }
            }
        });
    }

    fn start_scheduler(&self) {
        tracing::debug!("Start scheduler");
        let state = self.state.clone();
        let output = self.output.clone();

        std::thread::spawn(move || {
            loop {
                {
                    let now = Instant::now();
                    let mut tasks = vec![];
                    while let Some((instant, update)) = state.scheduled().pop() {
                        if now >= instant {
                            state.update(update, &output);
                        } else {
                            tasks.push((instant, update))
                        }
                    }
                    *state.scheduled() = tasks;
                }

                // TODO: This is a very basic system of scheduler ...
                std::thread::sleep(Duration::from_millis(10));
            }
        });
    }
}
