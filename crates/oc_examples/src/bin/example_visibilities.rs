#[cfg(feature = "test")]
use std::time::{Duration, Instant};
use std::{
    path::PathBuf,
    sync::atomic::{AtomicBool, Ordering},
};

use bevy::prelude::*;

use anyhow::Context;
use clap::{Parser, ValueEnum};
#[cfg(feature = "test")]
use oc_battle_gui::{
    entity::individual::IndividualIndex, ingame::individual::Gesture, states::Game,
};
use oc_examples::{logging, run, snapshot::SnapshotBuilder};
use oc_individual::order::Order;
use oc_root::{
    WorldConfig,
    geo::{WorldVec2, WorldVec3},
    physics::Meters,
    side,
};
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
            | TestCase::Hidden
            | TestCase::MoveThenEnemyVisible => Duration::from_secs(10),
            TestCase::Discover => Duration::from_secs(20),
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum TestCase {
    Direct,
    Through,
    Hidden,
    Discover,
    MoveThenEnemyVisible,
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
    let w = WorldConfig::new(
        map_.width().unwrap() as u64,
        map_.height().unwrap() as u64,
        Meters(meta.geo_meters_per_z),
    );
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
                .make(&w),
            TestIndividual::builder()
                .side(side::Side::B)
                .position(WorldVec3::new(250., 150., 0.))
                .build()
                .make(&w),
        ],
        TestCase::Through => vec![
            TestIndividual::builder()
                .side(side::Side::A)
                .position(WorldVec3::new(250., 250., 0.))
                .build()
                .make(&w),
            TestIndividual::builder()
                .side(side::Side::B)
                .position(WorldVec3::new(450., 250., 0.))
                .build()
                .make(&w),
        ],
        TestCase::Hidden => vec![
            TestIndividual::builder()
                .side(side::Side::A)
                .position(WorldVec3::new(250., 250., 0.))
                .build()
                .make(&w),
            TestIndividual::builder()
                .side(side::Side::B)
                .position(WorldVec3::new(450., 150., 0.))
                .build()
                .make(&w),
        ],
        TestCase::Discover => vec![
            TestIndividual::builder()
                .side(side::Side::A)
                .position(WorldVec3::new(250., 175., 0.))
                .build()
                .make(&w),
            TestIndividual::builder()
                .side(side::Side::B)
                .position(WorldVec3::new(450., 150., 0.))
                .build()
                .make(&w),
        ],
        TestCase::MoveThenEnemyVisible => vec![
            TestIndividual::builder()
                .side(side::Side::A)
                .position(WorldVec3::new(250., 175., 0.))
                .build()
                .make(&w),
            TestIndividual::builder()
                .side(side::Side::B)
                .position(WorldVec3::new(450., 150., 0.))
                .build()
                .make(&w),
        ],
    }
}

fn squads(
    _w: &WorldConfig,
    _tiles: &Vec<Tile>,
    individuals: &Vec<oc_individual::Individual>,
    args: &Args,
) -> Vec<oc_individual::squad::Squad> {
    match args.case {
        TestCase::Direct | TestCase::Through | TestCase::Hidden => vec![
            TestSquad::builder()
                .position(individuals.get(0).unwrap().position.into())
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
                .position(individuals.get(0).unwrap().position.into())
                .members(vec![oc_individual::IndividualIndex(0)])
                .orders(vec![Order::MoveFastTo(WorldVec2::new(250., 150.))])
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
                .position(individuals.get(0).unwrap().position.into())
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
        | TestCase::Hidden
        | TestCase::Discover
        | TestCase::MoveThenEnemyVisible => {
            #[cfg(feature = "test")]
            app.add_systems(Update, tracking);
        }
    }
}

#[cfg(feature = "test")]
fn test_tracker(mut commands: Commands, game: Res<Game>, state: ResMut<State>) {
    let args = Args::parse();
    let timeout = args.timeout();
    if game.started.elapsed() > timeout
        || state
            .success
            .map(|success| success.elapsed() > AFTER_SUCCESS_WAIT)
            .unwrap_or_default()
    {
        commands.write_message(bevy::app::AppExit::from_code(0));
    }
}

#[cfg(feature = "test")]
fn tracking(mut state: ResMut<State>, query: Query<(&IndividualIndex, &Visibility, &Gesture)>) {
    let args = Args::parse();

    let i1_visible = query
        .iter()
        .any(|(i, v, _)| i.0 == oc_individual::IndividualIndex(0) && v == &Visibility::Visible);
    let i1_gesture = query
        .iter()
        .filter_map(|(i, _, g)| (i.0 == oc_individual::IndividualIndex(0)).then(|| &g.0))
        .next();
    let i2_visible = query
        .iter()
        .any(|(i, v, _)| i.0 == oc_individual::IndividualIndex(1) && v == &Visibility::Visible);
    let i2_gesture = query
        .iter()
        .filter_map(|(i, _, g)| (i.0 == oc_individual::IndividualIndex(1)).then(|| &g.0))
        .next();

    if match args.case {
        TestCase::Direct => {
            i1_visible
                && i2_visible
                && matches!(i1_gesture, Some(&oc_individual::Gesture::Prone(_)))
                && matches!(i2_gesture, Some(&oc_individual::Gesture::Prone(_)))
        }
        TestCase::Through => {
            i1_visible
                && i2_visible
                && matches!(i1_gesture, Some(&oc_individual::Gesture::Prone(_)))
                && matches!(i2_gesture, Some(&oc_individual::Gesture::Prone(_)))
        }
        TestCase::Hidden => {
            i1_visible
                && !i2_visible
                && matches!(i1_gesture, Some(&oc_individual::Gesture::StandUp(_)))
                && matches!(i2_gesture, Some(&oc_individual::Gesture::StandUp(_)))
        }
        TestCase::Discover => {
            i1_visible
                && i2_visible
                && matches!(i1_gesture, Some(&oc_individual::Gesture::Prone(_)))
                && matches!(i2_gesture, Some(&oc_individual::Gesture::Prone(_)))
        }
        TestCase::MoveThenEnemyVisible => {
            matches!(i1_gesture, Some(&oc_individual::Gesture::Running(_)))
        }
    } {
        // FIXME: must test individuals behavior/gesture too (hide)
        state.success = Some(Instant::now());
        SUCCESS.store(true, Ordering::Relaxed);
    }
}
