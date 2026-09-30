#![allow(clippy::unwrap_used)]

use std::{
    path::PathBuf,
    sync::atomic::{AtomicBool, Ordering},
};
#[cfg(feature = "test")]
use std::{
    sync::Mutex,
    time::{Duration, Instant},
};

use bevy::prelude::*;

use anyhow::Context;
use clap::{Parser, ValueEnum};
#[cfg(feature = "test")]
use oc_battle_gui::{
    entity::individual::IndividualIndex,
    ingame::individual::{Gesture, SetGestureEvent},
    states::{Game, GameConfig},
};
use oc_examples::{logging, run, snapshot::SnapshotBuilder};
use oc_individual::order::Order;
#[cfg(feature = "test")]
use oc_root::Wcfg;
use oc_root::{
    WorldConfig,
    geo::{WorldVec2, WorldVec3},
    physics::Meters,
    side,
};
#[cfg(feature = "test")]
use oc_utils::let_some;
use oc_world::{meta::Meta, tile::Tile};
use tests::{individual::TestIndividual, squad::TestSquad};

#[cfg(feature = "test")]
const AFTER_SUCCESS_WAIT: Duration = Duration::from_secs(1);
const MOD: &str = "mods/tests1";

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg()]
    case: TestCase,

    #[arg(long, action)]
    test: bool,
}

#[cfg(feature = "test")]
impl Args {
    fn timeout(&self) -> Duration {
        match self.case {
            TestCase::Direct
            | TestCase::Through
            | TestCase::NotVisible
            | TestCase::MoveThenEnemyVisible
            | TestCase::Hidden => Duration::from_secs(10),
            TestCase::Discover => Duration::from_secs(20),
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum TestCase {
    Direct,
    Through,
    NotVisible,
    Discover,
    MoveThenEnemyVisible,
    // FIXME BS NOW: implement hiding bonuses:
    //   - visibility when immobile
    //   - visibility when crawling ?
    // FIXME BS NOW 2: when fire on
    // FIXME BS NOW 3: tile nature should sometime (brick wall 90%, trunk 70%, etc) "block" projectile
    Hidden,
}

fn main() -> Result<(), anyhow::Error> {
    logging::setup_logging()?;

    let args = Args::parse();
    if args.test {
        #[cfg(not(feature = "test"))]
        {
            panic!("To enable test, feature `test` must be enabled too")
        }
    }

    let mod_ = PathBuf::from(MOD);
    let mod__ = oc_mod::Mod::load(&mod_, None)?;
    let map = PathBuf::from("examples/meadow1");
    let meta = Meta::from_file(&map.join("meta.toml"))?;
    let map_ = oc_world::reader::MapReader::new(&map);
    let map_ = map_.context(format!("Read map_ {}", map.display()))?;
    let w = WorldConfig::new(map_.width().unwrap() as u64, map_.height().unwrap() as u64)
        .with_geo_meters_per_z(Meters(meta.geo_meters_per_z));
    let tiles = map_.tiles(&w, &mod__).unwrap();

    let individuals = individuals(&w, &tiles, &args);
    let squads = squads(&w, &tiles, &individuals, &args);
    let snapshot = SnapshotBuilder::new(map_, individuals, squads, vec![]).build(w, &mod__)?;

    let example = run::Example::builder()
        .world(map)
        .mod_(mod_)
        .install(Box::new(install))
        .snapshot(snapshot)
        .test_app_exit_code(args.test);

    example.build().run()?;

    if args.test {
        if !SUCCESS.load(Ordering::Relaxed) {
            anyhow::bail!("❌ Test failed")
        }
        println!("✅ Test success !");
    }

    Ok(())
}

fn individuals(w: &WorldConfig, _tiles: &Vec<Tile>, args: &Args) -> Vec<oc_individual::Individual> {
    match args.case {
        TestCase::Direct => vec![
            TestIndividual::builder()
                .side(side::Side::A)
                .position(WorldVec3::new(250., 250., 0.))
                .build()
                .make(w),
            TestIndividual::builder()
                .side(side::Side::B)
                .position(WorldVec3::new(250., 150., 0.))
                .build()
                .make(w),
        ],
        TestCase::Through => vec![
            TestIndividual::builder()
                .side(side::Side::A)
                .position(WorldVec3::new(250., 250., 0.))
                .build()
                .make(w),
            TestIndividual::builder()
                .side(side::Side::B)
                .position(WorldVec3::new(450., 250., 0.))
                .build()
                .make(w),
        ],
        TestCase::NotVisible => vec![
            TestIndividual::builder()
                .side(side::Side::A)
                .position(WorldVec3::new(250., 250., 0.))
                .build()
                .make(w),
            TestIndividual::builder()
                .side(side::Side::B)
                .position(WorldVec3::new(450., 150., 0.))
                .build()
                .make(w),
        ],
        TestCase::Discover => vec![
            TestIndividual::builder()
                .side(side::Side::A)
                .position(WorldVec3::new(250., 225., 0.))
                .build()
                .make(w),
            TestIndividual::builder()
                .side(side::Side::B)
                .position(WorldVec3::new(450., 140., 0.))
                .build()
                .make(w),
        ],
        TestCase::MoveThenEnemyVisible => vec![
            TestIndividual::builder()
                .side(side::Side::A)
                .position(WorldVec3::new(250., 225., 0.))
                .build()
                .make(w),
            TestIndividual::builder()
                .side(side::Side::B)
                .position(WorldVec3::new(450., 140., 0.))
                .build()
                .make(w),
        ],
        TestCase::Hidden => vec![
            TestIndividual::builder()
                .side(side::Side::A)
                .position(WorldVec3::new(25., 224., 0.))
                .build()
                .make(w),
            TestIndividual::builder()
                .side(side::Side::B)
                .position(WorldVec3::new(387., 198., 0.))
                .build()
                .make(w),
        ],
    }
}

fn squads(
    _w: &WorldConfig,
    _tiles: &Vec<Tile>,
    individuals: &[oc_individual::Individual],
    args: &Args,
) -> Vec<oc_individual::squad::Squad> {
    match args.case {
        TestCase::Direct | TestCase::Through | TestCase::NotVisible | TestCase::Hidden => vec![
            TestSquad::builder()
                .position(individuals.first().unwrap().position.into())
                .members(vec![oc_individual::IndividualIndex(0)])
                .orders(vec![])
                .build()
                .make(),
            TestSquad::builder()
                .position(individuals.get(1).unwrap().position.into())
                .members(vec![oc_individual::IndividualIndex(1)])
                .orders(vec![])
                .build()
                .make(),
        ],
        TestCase::Discover => vec![
            TestSquad::builder()
                .position(individuals.first().unwrap().position.into())
                .members(vec![oc_individual::IndividualIndex(0)])
                .orders(vec![Order::MoveFastTo(WorldVec2::new(250., 110.))])
                .build()
                .make(),
            TestSquad::builder()
                .position(individuals.get(1).unwrap().position.into())
                .members(vec![oc_individual::IndividualIndex(1)])
                .orders(vec![])
                .build()
                .make(),
        ],
        TestCase::MoveThenEnemyVisible => vec![
            TestSquad::builder()
                .position(individuals.first().unwrap().position.into())
                .members(vec![oc_individual::IndividualIndex(0)])
                .orders(vec![Order::MoveTo(WorldVec2::new(250., 100.))])
                .build()
                .make(),
            TestSquad::builder()
                .position(individuals.get(1).unwrap().position.into())
                .members(vec![oc_individual::IndividualIndex(1)])
                .orders(vec![])
                .build()
                .make(),
        ],
    }
}

static SUCCESS: AtomicBool = AtomicBool::new(false);

#[cfg(feature = "test")]
#[derive(Debug, Resource, Default)]
struct State {
    success: Option<Instant>,
    gestures: Vec<(oc_individual::IndividualIndex, oc_individual::Gesture)>,
}

#[allow(unused)]
fn install(app: &mut bevy::app::App) {
    let args = Args::parse();

    #[cfg(feature = "test")]
    if args.test {
        app.add_systems(Update, test_tracker);
    }

    #[cfg(feature = "test")]
    app.init_resource::<State>();

    match args.case {
        TestCase::Direct
        | TestCase::Through
        | TestCase::NotVisible
        | TestCase::Discover
        | TestCase::MoveThenEnemyVisible
        | TestCase::Hidden => {
            #[cfg(feature = "test")]
            app.add_systems(Update, tracking)
                .add_observer(on_set_gesture);
        }
    }
}

#[cfg(feature = "test")]
fn on_set_gesture(event: On<SetGestureEvent>, mut state: ResMut<State>) {
    state.gestures.push((event.0, event.1.clone()));
}

#[cfg(feature = "test")]
fn test_tracker(mut commands: Commands, game: Res<Game>, state: ResMut<State>, w: Res<Wcfg>) {
    let_some!(w = &w.0, return);
    let args = Args::parse();
    let speed = w.speed();
    // Ensure a minimal timeout to let time to bevy to make "network" exchange with server
    let timeout = args.timeout().div_f32(speed).max(Duration::from_secs(2));
    if game.started.elapsed() > timeout
        || state
            .success
            .map(|success| success.elapsed() >= AFTER_SUCCESS_WAIT.div_f32(speed))
            .unwrap_or_default()
    {
        commands.write_message(bevy::app::AppExit::from_code(0));
    }
}

#[cfg(feature = "test")]
fn tracking(
    mut state: ResMut<State>,
    query: Query<(&IndividualIndex, &Visibility, &Gesture)>,
    g: Res<GameConfig>,
) {
    let_some!(g = &g.0, return);
    let args = Args::parse();

    static I2_INVISIBLE_SINCE: Mutex<Option<Instant>> = Mutex::new(None);

    let i1_visible = query
        .iter()
        .any(|(i, v, _)| i.0 == oc_individual::IndividualIndex(0) && v == Visibility::Visible);
    let i1_gesture = query
        .iter()
        .filter_map(|(i, _, g)| (i.0 == oc_individual::IndividualIndex(0)).then_some(&g.0.body))
        .next();
    let i2_visible = query
        .iter()
        .any(|(i, v, _)| i.0 == oc_individual::IndividualIndex(1) && v == Visibility::Visible);
    let i2_gesture = query
        .iter()
        .filter_map(|(i, _, g)| (i.0 == oc_individual::IndividualIndex(1)).then_some(&g.0.body))
        .next();

    if !i2_visible && I2_INVISIBLE_SINCE.lock().unwrap().is_none() {
        *I2_INVISIBLE_SINCE.lock().unwrap() = Some(Instant::now());
    }

    if match args.case {
        TestCase::Direct | TestCase::Through => {
            i1_visible
                && i2_visible
                && matches!(i1_gesture, Some(&oc_individual::BodyGesture::Prone(_)))
                && matches!(i2_gesture, Some(&oc_individual::BodyGesture::Prone(_)))
        }
        TestCase::NotVisible => {
            i1_visible
                && !i2_visible
                && matches!(i1_gesture, Some(&oc_individual::BodyGesture::StandUp(_)))
                && matches!(i2_gesture, Some(&oc_individual::BodyGesture::StandUp(_)))
        }
        TestCase::Discover => {
            i1_visible
                && i2_visible
                && matches!(i1_gesture, Some(&oc_individual::BodyGesture::Prone(_)))
                && matches!(i2_gesture, Some(&oc_individual::BodyGesture::Prone(_)))
        }
        TestCase::MoveThenEnemyVisible => state.gestures.iter().any(|(i, gesture)| {
            *i == oc_individual::IndividualIndex(0)
                && matches!(gesture.body, oc_individual::BodyGesture::Prone(_))
        }),
        TestCase::Hidden => {
            i1_visible
                && !i2_visible
                // FIXME BS NOW: add never been seen (must modify source code to begin in correct gesture)
                && I2_INVISIBLE_SINCE
                    .lock()
                    .unwrap()
                    .is_some_and(|s| s.elapsed().as_secs_f32() >= 1. / g.w.speed())
        }
    } {
        // FIXME: must test individuals behavior/gesture too (hide)
        state.success = Some(Instant::now());
        SUCCESS.store(true, Ordering::Relaxed);
    }
}
