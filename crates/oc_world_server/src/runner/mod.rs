use std::{
    sync::{
        Arc,
        mpsc::{Receiver, Sender},
    },
    time::{Duration, Instant},
};

use derive_more::Constructor;
use oc_network::ToClient;
use oc_root::Client;
use rayon::{
    iter::{IndexedParallelIterator, ParallelIterator},
    slice::ParallelSlice,
};

#[cfg(feature = "tracker")]
use crate::tracker::Tracker;
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
    #[cfg(feature = "tracker")]
    tracker: Tracker,
}

impl<E: Client> Runner<E> {
    pub fn run(&self, input: Receiver<Event<E>>, ready: Sender<Result<(), String>>) {
        self.listen_input(input);
        self.start_physics();
        self.start_individuals();
        self.start_scheduler();

        let _ = ready.send(Ok(()));
        #[cfg(feature = "perfs")]
        self.perfs();

        tracing::debug!("Finished runner");
    }

    fn start_physics(&self) {
        tracing::debug!("Start physics");
        let ctx = Context::new(
            self.state.clone(),
            self.output.clone(),
            #[cfg(feature = "tracker")]
            self.tracker.clone(),
        );

        #[cfg(feature = "perfs")]
        {
            *ctx.state
                .perf
                .physic_percents
                .lock()
                .expect("Assume available") = vec![0.; ctx.cpus];
        }
        let interval = ctx.state.w.physics_tick_interval_us;

        (0..ctx.cpus).for_each(|i| {
            let ctx = ctx.clone();

            std::thread::spawn(move || {
                let mut last = Instant::now();

                loop {
                    let elapsed = last.elapsed().as_micros() as u64;
                    let wait = interval - elapsed.min(interval);
                    #[cfg(feature = "perfs")]
                    {
                        let percent = wait as f32 / interval as f32;
                        ctx.state.perf.set_physic_percent(i, 1. - percent);
                    }

                    std::thread::sleep(Duration::from_micros(wait));
                    last = Instant::now();

                    for update in physics::Processor::new(&ctx).step(i) {
                        ctx.state.update(update, &ctx.output);
                    }
                }
            });
        });
    }

    fn start_individuals(&self) {
        tracing::debug!("Start individuals");

        let ctx = Context::new(
            self.state.clone(),
            self.output.clone(),
            #[cfg(feature = "tracker")]
            self.tracker.clone(),
        );
        let individuals_count = {
            let world = self.state.world();
            world.individuals().len()
        };
        let size = (individuals_count as f32 / ctx.cpus as f32).ceil() as usize;
        if size == 0 {
            return;
        }

        #[cfg(feature = "perfs")]
        {
            *ctx.state
                .perf
                .individual_percents
                .lock()
                .expect("Assume available") = vec![0.; ctx.cpus];
        }
        let interval = ctx.state.w.individual_tick_interval_us;

        (0..individuals_count)
            .collect::<Vec<usize>>()
            .par_chunks(size)
            .enumerate()
            .for_each(|(_i, indexes)| {
                let indexes = indexes.to_vec();
                let ctx = ctx.clone();

                std::thread::spawn(move || {
                    let mut last = Instant::now();

                    loop {
                        let elapsed = last.elapsed().as_micros() as u64;
                        let wait = interval - elapsed.max(interval);
                        #[cfg(feature = "perfs")]
                        {
                            let percent = wait as f32 / interval as f32;
                            ctx.state.perf.set_individual_percent(_i, 1. - percent);
                        }

                        std::thread::sleep(Duration::from_micros(wait));
                        last = Instant::now();

                        for i in &indexes {
                            tracing::trace!(name = "runner-individual", i = ?i);
                            individual::Processor::new(&ctx, (*i).into()).step();

                            #[cfg(feature = "perfs")]
                            ctx.state.perf.increment_individual();
                        }
                    }
                });
            });
    }

    #[cfg(feature = "perfs")]
    fn perfs(&self) {
        tracing::debug!("Track perfs");
        loop {
            std::thread::sleep(Duration::from_secs(1));

            let individuals = self.state.world.read().unwrap().individuals().len();
            let projectiles = self.state.world.read().unwrap().projectiles().len();
            let individual_percents = self
                .state
                .perf
                .individual_percents
                .lock()
                .expect("Assume available")
                .iter()
                .map(|percent| format!("{percent:.3}"))
                .collect::<Vec<String>>()
                .join(",");
            let physic_percents = self
                .state
                .perf
                .physic_percents
                .lock()
                .expect("Assume available")
                .iter()
                .map(|percent| format!("{percent:.3}"))
                .collect::<Vec<String>>()
                .join(",");
            let physics = projectiles;

            println!(
                "individuals({individuals}): {} tick/s({}); physics({physics}): {} tick/s({})",
                self.state.perf.individuals_ticks(),
                individual_percents,
                self.state.perf.physics_ticks(),
                physic_percents,
            );

            self.state.perf.reset();
        }
    }

    fn listen_input(&self, input: Receiver<Event<E>>) {
        tracing::debug!("Listen inputs");
        let state = self.state.clone();
        let output = self.output.clone();
        let w = self.state.w.clone();
        let mod_ = self.state.world().mod_().clone();
        let meta = self.state.world().meta().clone();
        let static_ = self.config.static_.clone();

        std::thread::spawn(move || {
            while let Ok(message) = input.recv() {
                match message {
                    Event::Connected(endpoint) => {
                        state.listeners_mut().push(endpoint.clone());
                        let w = ToClient::Wcfg(w.clone());
                        output.send((endpoint.clone(), w)).unwrap(); // TODO
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
