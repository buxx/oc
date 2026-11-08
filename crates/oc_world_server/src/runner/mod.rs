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

#[cfg(feature = "tracker")]
use crate::tracker::Tracker;
use crate::{
    bridge::Event, config::ServerConfig, individual, physics, runner::input::Dealer, squad,
    state::State, utils::context::Context,
};

const WORLD_INTERVAL: Duration = Duration::from_secs(5);
/// Smoke clouds fading precision
const SMOKE_INTERVAL: Duration = Duration::from_millis(250);
const SMOKE_MIN_INTERVAL: Duration = Duration::from_millis(20); // For e2e tests purposes
/// Watchers computed by visibilities thread between two releases of world locks
const VISIBILITIES_CHUNK: usize = 2;
/// Individuals/squads step
const STEP_BATCH: usize = 16;

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
        self.start_smoke();
        self.start_scheduler();

        let _ = ready.send(Ok(()));
        #[cfg(feature = "perfs")]
        self.perfs();

        let _ = stop.recv();
        tracing::debug!("Finished runner");
    }

    fn context(&self) -> Context<E> {
        Context::new(
            self.state.clone(),
            self.output.clone(),
            #[cfg(feature = "tracker")]
            self.tracker.clone(),
            #[cfg(feature = "debug")]
            self.direct.clone(),
        )
    }

    fn start_physics(&self) {
        tracing::debug!("Start physics");
        let ctx = self.context();

        (0..ctx.cpus).for_each(|i| {
            spawn_ticker(ctx.clone(), Kind::Physics, true, move |ctx| {
                tracing::trace!(name = "runner-physics", i = i);
                for update_ in physics::Processor::new(ctx).step(i) {
                    update::update(ctx, update_);
                }
            });
        });
    }

    fn start_individuals(&self) {
        tracing::debug!("Start individuals");
        let ctx = self.context();
        let count = self.state.world().individuals().len();

        spawn_chunked(ctx, Kind::Individuals, count, |ctx, batch| {
            let world = &ctx.state.world();
            let indexes = &ctx.state.indexes();
            batch
                .iter()
                .flat_map(|i| {
                    let i = IndividualIndex(*i as u64);
                    tracing::trace!(name = "runner-individual", i = ?i);
                    individual::Processor::new(world, indexes, i).step()
                })
                .collect::<Vec<_>>()
        });
    }

    fn start_squads(&self) {
        tracing::debug!("Start squads");
        let ctx = self.context();
        let count = self.state.world().squads().len();

        spawn_chunked(ctx, Kind::Squads, count, |ctx, batch| {
            let world = &ctx.state.world();
            let indexes = &ctx.state.indexes();
            batch
                .iter()
                .flat_map(|i| {
                    let i = SquadIndex(*i as u64);
                    tracing::trace!(name = "runner-squad", i = ?i);
                    squad::Processor::new(world, indexes, i).step()
                })
                .collect::<Vec<_>>()
        });
    }

    fn start_visibilities(&self) {
        tracing::debug!("Start visibilities");
        let ctx = self.context();
        if self.state.world().individuals().is_empty() {
            return;
        }

        spawn_ticker(ctx, Kind::Visibilities, false, |ctx| {
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
                    return;
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

            update::update(ctx, update::Update::UpdateVisibilities(visibilities));
        });
    }

    fn start_world(&self) {
        tracing::debug!("Start world");

        spawn_ticker(self.context(), Kind::World, true, |ctx| {
            let updates = {
                let world = &ctx.state.world();
                tracing::trace!(name = "runner-world");

                crate::world::Processor::new(world).compute()
            };

            for update in updates {
                update::update(ctx, update);
            }
        });
    }

    fn start_smoke(&self) {
        tracing::debug!("Start smoke");

        spawn_ticker(self.context(), Kind::Smoke, true, |ctx| {
            tracing::trace!(name = "runner-smoke");
            let has_clouds = !ctx.state.world().smoke_clouds.is_empty();
            if has_clouds {
                update::update(ctx, update::Update::UpdateSmoke);
            }
        });
    }

    #[cfg(feature = "perfs")]
    fn perfs(&self) {
        let state = self.state.clone();
        std::thread::spawn(move || {
            tracing::debug!("Track perfs");
            loop {
                std::thread::sleep(std::time::Duration::from_secs(1));

                let loads = state
                    .perf
                    .take()
                    .iter()
                    .map(|(name, percent)| format!("{name}: {percent:.0}%"))
                    .collect::<Vec<String>>()
                    .join(", ");
                println!("load: {loads}");
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

        let ctx = self.context();

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
        let ctx = self.context();

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
    matches!(ctx.state.world().phase, BattlePhase::End(_))
}

/// Periodic runner kinds: each one has its own tick interval and perf load
#[derive(Clone, Copy)]
enum Kind {
    Physics,
    Individuals,
    Squads,
    Visibilities,
    World,
    Smoke,
}

impl Kind {
    fn interval<E: Client>(self, ctx: &Context<E>) -> Duration {
        let w = &ctx.state.w;
        match self {
            Kind::Physics => w.physics_tick().interval(),
            Kind::Individuals => w.individual_tick().interval(),
            Kind::Squads => w.squad_tick().interval(),
            Kind::Visibilities => w.visibilities_tick().interval(),
            Kind::World => WORLD_INTERVAL.div_f32(w.speed()),
            Kind::Smoke => SMOKE_INTERVAL.div_f32(w.speed()).max(SMOKE_MIN_INTERVAL),
        }
    }

    #[cfg(feature = "perfs")]
    fn load(self, perf: &crate::perf::Perf) -> &crate::perf::Load {
        match self {
            Kind::Physics => &perf.physics,
            Kind::Individuals => &perf.individuals,
            Kind::Squads => &perf.squads,
            Kind::Visibilities => &perf.visibilities,
            Kind::World | Kind::Smoke => &perf.world,
        }
    }
}

/// Spawn a thread calling `tick` once per `kind` interval. When `skip_ended` is set, ticks are
/// skipped once the battle is ended.
fn spawn_ticker<E: Client>(
    ctx: Context<E>,
    kind: Kind,
    skip_ended: bool,
    mut tick: impl FnMut(&Context<E>) + Send + 'static,
) {
    std::thread::spawn(move || {
        let mut last = Instant::now();

        loop {
            let interval = kind.interval(&ctx);
            let wait = interval - last.elapsed().min(interval);
            #[cfg(feature = "perfs")]
            kind.load(&ctx.state.perf).record(wait, interval);

            std::thread::sleep(wait);
            last = Instant::now();

            if skip_ended && ended(&ctx) {
                continue;
            }

            tick(&ctx);
        }
    });
}

/// Split `0..count` between one ticker thread by cpu. Each tick, `step` is called on batches of
/// indexes (locks are released between batches) and its updates applied.
fn spawn_chunked<E: Client>(
    ctx: Context<E>,
    kind: Kind,
    count: usize,
    step: impl Fn(&Context<E>, &[usize]) -> Vec<update::Update> + Clone + Send + 'static,
) {
    let size = (count as f32 / ctx.cpus as f32).ceil() as usize;
    if size == 0 {
        return;
    }

    for indexes in (0..count).collect::<Vec<usize>>().chunks(size) {
        let indexes = indexes.to_vec();
        let step = step.clone();

        spawn_ticker(ctx.clone(), kind, true, move |ctx| {
            for batch in indexes.chunks(STEP_BATCH) {
                update::update_all(ctx, step(ctx, batch));
            }
        });
    }
}
