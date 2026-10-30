use std::{
    sync::{
        Arc,
        mpsc::{Receiver, Sender},
    },
    time::{Duration, Instant},
};

use derive_more::Constructor;
use oc_individual::{IndividualIndex, squad::SquadIndex};
use oc_network::{GameConfig, ToClient};
#[cfg(feature = "debug")]
use oc_root::side::Side;
use oc_root::{Client, battle::BattlePhase};
use rayon::{
    iter::{IndexedParallelIterator, ParallelIterator},
    slice::ParallelSlice,
};

#[cfg(feature = "tracker")]
use crate::tracker::Tracker;
use crate::{
    bridge::Event, config::ServerConfig, individual, physics, runner::input::Dealer, squad,
    state::State, utils::context::Context,
};

const WORLD_INTERVAL: Duration = Duration::from_secs(5);
/// Watchers computed by visibilities thread between two releases of world locks
const VISIBILITIES_CHUNK: usize = 16;

pub mod explosion;
pub mod input;
pub mod support;
pub mod update;

#[derive(Constructor)]
pub struct Runner<E: Client> {
    config: ServerConfig,
    state: Arc<State<E>>,
    output: Sender<(E, ToClient)>,
    #[cfg(feature = "tracker")]
    tracker: Tracker,
    #[cfg(feature = "debug")]
    /// When set, avoid listening filter when broadcast (to ensure vent receive in e2e tests)
    direct: Option<(E, Side)>,
}

impl<E: Client> Runner<E> {
    pub fn run(
        &self,
        input: Receiver<Event<E>>,
        ready: Sender<Result<(), String>>,
        stop: Receiver<()>,
    ) {
        self.listen_input(input);
        self.start_physics();
        self.start_individuals();
        self.start_visibilities();
        self.start_squads();
        self.start_world();
        self.start_scheduler();

        let _ = ready.send(Ok(()));
        #[cfg(feature = "perfs")]
        self.perfs();

        let _ = stop.recv();
        tracing::debug!("Finished runner");
    }

    fn start_physics(&self) {
        tracing::debug!("Start physics");
        let ctx = Context::new(
            self.state.clone(),
            self.output.clone(),
            #[cfg(feature = "tracker")]
            self.tracker.clone(),
            #[cfg(feature = "debug")]
            self.direct.clone(),
        );

        #[cfg(feature = "perfs")]
        {
            ctx.state.perf.init_load("physics", ctx.cpus);
        }

        (0..ctx.cpus).for_each(|i| {
            let ctx = ctx.clone();

            std::thread::spawn(move || {
                let mut last = Instant::now();

                loop {
                    let elapsed = last.elapsed();
                    let interval = ctx.state.w.physics_tick().interval();
                    let wait = interval - elapsed.min(interval);

                    #[cfg(feature = "perfs")]
                    {
                        ctx.state.perf.record_load("physics", i, wait, interval);
                    }

                    tracing::trace!(
                        name = "runner-physics-sleep",
                        i = i,
                        wait = ?wait
                    );
                    std::thread::sleep(wait);
                    last = Instant::now();

                    if ended(&ctx) {
                        continue;
                    }

                    for update_ in physics::Processor::new(&ctx).step(i) {
                        update::update(&ctx, update_);
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
            #[cfg(feature = "debug")]
            self.direct.clone(),
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
            ctx.state.perf.init_load("individuals", ctx.cpus);
        }

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
                        let elapsed = last.elapsed();
                        let interval = ctx.state.w.individual_tick().interval();
                        let wait = interval - elapsed.min(interval);
                        #[cfg(feature = "perfs")]
                        {
                            ctx.state
                                .perf
                                .record_load("individuals", _i, wait, interval);
                        }

                        std::thread::sleep(wait);
                        last = Instant::now();

                        if ended(&ctx) {
                            continue;
                        }

                        for i in &indexes {
                            let i = IndividualIndex(*i as u64);
                            tracing::trace!(name = "runner-individual", i = ?i);

                            for update in {
                                // FIXME: optimize tensions on locks (wan, update make deadlock)
                                let world = &ctx.state.world();
                                let indexes = &ctx.state.indexes();
                                individual::Processor::new(world, indexes, i).step()
                            } {
                                update::update(&ctx, update);
                            }

                            #[cfg(feature = "perfs")]
                            ctx.state.perf.increment_individual();
                        }
                    }
                });
            });
    }
    fn start_visibilities(&self) {
        tracing::debug!("Start visibilities");

        let ctx = Context::new(
            self.state.clone(),
            self.output.clone(),
            #[cfg(feature = "tracker")]
            self.tracker.clone(),
            #[cfg(feature = "debug")]
            self.direct.clone(),
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
            ctx.state.perf.init_load("visibilities", 1);
        }
        let ctx = ctx.clone();

        std::thread::spawn(move || {
            let mut last = Instant::now();

            loop {
                let elapsed = last.elapsed();
                let interval = ctx.state.w.visibilities_tick().interval();
                let wait = interval - elapsed.min(interval);
                #[cfg(feature = "perfs")]
                {
                    ctx.state
                        .perf
                        .record_load("visibilities", 0, wait, interval);
                }

                tracing::trace!(name = "runner-visibilities-sleep", wait=?wait);
                std::thread::sleep(wait);
                last = Instant::now();

                let visibilities = {
                    // Short lock to start: sides individuals
                    let sides = {
                        let world = &ctx.state.world();
                        let indexes = &ctx.state.indexes();
                        tracing::trace!(name = "runner-visibilities");

                        matches!(world.phase, BattlePhase::Fight).then(|| {
                            (
                                indexes.side_a_individuals().to_vec(),
                                indexes.side_b_individuals().to_vec(),
                            )
                        })
                    };
                    let Some((side_a, side_b)) = sides else {
                        continue;
                    };

                    // Then compute by chunks of watchers, releasing locks between chunks to let
                    // writers (individuals, physics, ...) go through
                    let mut visibilities = vec![];
                    for (watchers, targets) in [(&side_a, &side_b), (&side_b, &side_a)] {
                        for chunk in watchers.chunks(VISIBILITIES_CHUNK) {
                            let world = &ctx.state.world();
                            let indexes = &ctx.state.indexes();
                            visibilities.extend(
                                crate::visibility::Processor::new(world, indexes)
                                    .compute_for(chunk, targets),
                            );
                        }
                    }
                    visibilities
                };

                #[cfg(feature = "perfs")]
                ctx.state.perf.increment_visibilities();

                update::update(&ctx, update::Update::UpdateVisibilities(visibilities));
            }
        });
    }

    fn start_world(&self) {
        tracing::debug!("Start world");

        let ctx = Context::new(
            self.state.clone(),
            self.output.clone(),
            #[cfg(feature = "tracker")]
            self.tracker.clone(),
            #[cfg(feature = "debug")]
            self.direct.clone(),
        );

        #[cfg(feature = "perfs")]
        ctx.state.perf.init_load("world", 1);

        std::thread::spawn(move || {
            let mut last = Instant::now();

            loop {
                let elapsed = last.elapsed();
                let interval = WORLD_INTERVAL.div_f32(ctx.state.w.speed());
                let wait = interval - elapsed.min(interval);
                #[cfg(feature = "perfs")]
                ctx.state.perf.record_load("world", 0, wait, interval);

                tracing::trace!(name = "runner-world-sleep", wait=?wait);
                std::thread::sleep(wait);
                last = Instant::now();

                if ended(&ctx) {
                    continue;
                }

                let updates = {
                    let world = &ctx.state.world();
                    tracing::trace!(name = "runner-world");

                    crate::world::Processor::new(world).compute()
                };

                for update in updates {
                    update::update(&ctx, update);
                }
            }
        });
    }

    fn start_squads(&self) {
        tracing::debug!("Start squads");

        let ctx = Context::new(
            self.state.clone(),
            self.output.clone(),
            #[cfg(feature = "tracker")]
            self.tracker.clone(),
            #[cfg(feature = "debug")]
            self.direct.clone(),
        );
        let squads_count = {
            let world = self.state.world();
            world.squads().len()
        };
        let size = (squads_count as f32 / ctx.cpus as f32).ceil() as usize;
        if size == 0 {
            return;
        }

        #[cfg(feature = "perfs")]
        {
            ctx.state.perf.init_load("squads", ctx.cpus);
        }

        (0..squads_count)
            .collect::<Vec<usize>>()
            .par_chunks(size)
            .enumerate()
            .for_each(|(_i, indexes)| {
                let indexes = indexes.to_vec();
                let ctx = ctx.clone();

                std::thread::spawn(move || {
                    let mut last = Instant::now();

                    loop {
                        let elapsed = last.elapsed();
                        let interval = ctx.state.w.squad_tick().interval();
                        let wait = interval - elapsed.min(interval);
                        #[cfg(feature = "perfs")]
                        {
                            ctx.state.perf.record_load("squads", _i, wait, interval);
                        }

                        std::thread::sleep(wait);
                        last = Instant::now();

                        if ended(&ctx) {
                            continue;
                        }

                        for i in &indexes {
                            let i = SquadIndex(*i as u64);
                            tracing::trace!(name = "runner-squad", i = ?i);

                            for update in {
                                let world = &ctx.state.world();
                                let indexes = &ctx.state.indexes();
                                squad::Processor::new(world, indexes, i).step()
                            } {
                                update::update(&ctx, update);
                            }

                            #[cfg(feature = "perfs")]
                            ctx.state.perf.increment_squad();
                        }
                    }
                });
            });
    }

    #[cfg(feature = "perfs")]
    fn perfs(&self) {
        let state = self.state.clone();
        std::thread::spawn(move || {
            tracing::debug!("Track perfs");
            loop {
                std::thread::sleep(std::time::Duration::from_secs(1));

                let individuals = state.world.read().unwrap().individuals().len();
                let projectiles = state.world.read().unwrap().projectiles().len();
                let loads = state
                    .perf
                    .take_loads()
                    .into_iter()
                    .map(|(name, percent)| format!("{name}: {percent:.0}%"))
                    .collect::<Vec<String>>()
                    .join(" ");

                println!(
                    "individuals({individuals}): {} tick/s; physics({projectiles}): {} tick/s; load: {loads}",
                    state.perf.individuals_ticks(),
                    state.perf.physics_ticks(),
                );

                state.perf.reset();
            }
        });
    }

    fn listen_input(&self, input: Receiver<Event<E>>) {
        tracing::debug!("Listen inputs");
        let state = self.state.clone();
        let output = self.output.clone();
        let w = self.state.w.clone();
        let mod_ = self.state.world().mod_().clone();
        let meta = self.state.world().meta().clone();
        let static_ = self.config.static_.clone();

        let ctx = Context::new(
            self.state.clone(),
            self.output.clone(),
            #[cfg(feature = "tracker")]
            self.tracker.clone(),
            #[cfg(feature = "debug")]
            self.direct.clone(),
        );

        std::thread::spawn(move || {
            while let Ok(message) = input.recv() {
                match message {
                    Event::Connected(endpoint) => {
                        state.listeners_mut().push(endpoint.clone());

                        let config =
                            GameConfig::new(w.clone(), mod_.clone(), meta.clone(), static_.clone());
                        output
                            .send((endpoint.clone(), ToClient::GameConfig(Box::new(config))))
                            .unwrap(); // TODO
                    }
                    Event::Disconnected(endpoint) => state.listeners_mut().remove(&endpoint),
                    Event::Message(endpoint, message) => {
                        for update_ in Dealer::new(&state, &mod_, &output, endpoint).deal(message) {
                            update::update(&ctx, update_);
                        }
                    }
                }
            }
        });
    }

    fn start_scheduler(&self) {
        tracing::debug!("Start scheduler");
        let ctx = Context::new(
            self.state.clone(),
            self.output.clone(),
            #[cfg(feature = "tracker")]
            self.tracker.clone(),
            #[cfg(feature = "debug")]
            self.direct.clone(),
        );

        std::thread::spawn(move || {
            loop {
                let now = Instant::now();
                // Take all tasks at once: lock must be released before run updates
                let tasks = std::mem::take(&mut *ctx.state.scheduled());
                let (due, later): (Vec<_>, Vec<_>) =
                    tasks.into_iter().partition(|(instant, _)| now >= *instant);
                ctx.state.scheduled().extend(later);
                for (_, update_) in due {
                    update::update(&ctx, update_);
                }

                // TODO: This is a very basic system of scheduler ...
                std::thread::sleep(ctx.state.w.scheduler_tick().interval());
            }
        });
    }
}

fn ended<E: Client>(ctx: &Context<E>) -> bool {
    matches!(ctx.state.world().phase, BattlePhase::End)
}
