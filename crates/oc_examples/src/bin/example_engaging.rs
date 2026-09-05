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
    entity::individual::IndividualIndex,
    ingame::individual::{Gesture, Status},
    states::Game,
};
use oc_examples::{logging, run, snapshot::SnapshotBuilder};
use oc_root::{WorldConfig, geo::WorldVec3, physics::Meters, side};
use oc_world::{meta::Meta, tile::Tile};
use tests::{
    individual::TestIndividual,
    squad::TestSquad,
    weapons::{TestWeapon, TestWeapons},
};

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

    /// All shots are precise (true when --test)
    #[arg(long, action)]
    precise: bool,
}

#[cfg(feature = "test")]
impl Args {
    fn timeout(&self) -> Duration {
        match self.case {
            TestCase::Direct | TestCase::Direct2 | TestCase::FarMachineGun => {
                Duration::from_secs(10)
            }
            TestCase::Suppressed => Duration::from_secs(30),
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum TestCase {
    Direct,
    Direct2,
    FarMachineGun,
    Suppressed,
}

// FIXME BS NOW: set random precision (fire) and permit set precise at 100% for test
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
    )
    .visibilities_tick_each_seconds(0.5); // To ensure one shot and test it

    let w = match args.case {
        TestCase::Direct | TestCase::Direct2 => w,
        TestCase::FarMachineGun | TestCase::Suppressed => match args.test {
            true => w.individual_tick_interval_us(1_000_000 / 10),
            false => w,
        },
    };

    let tiles = map_.tiles(&w, &mod__).unwrap();

    let individuals = individuals(&w, &mod__, &tiles, &args);
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

fn individuals(
    w: &WorldConfig,
    mod_: &oc_mod::Mod,
    _tiles: &Vec<Tile>,
    args: &Args,
) -> Vec<oc_individual::Individual> {
    match args.case {
        TestCase::Direct => vec![
            TestIndividual::builder()
                .side(side::Side::A)
                .position(WorldVec3::new(250., 250., 0.))
                .weapons(
                    TestWeapons::builder()
                        .primary(TestWeapon::filled(mod_, "Weapon3").make())
                        .build()
                        .make(),
                )
                .build()
                .make(&w),
            TestIndividual::builder()
                .side(side::Side::B)
                .position(WorldVec3::new(250., 150., 0.))
                .build()
                .make(&w),
        ],
        TestCase::Direct2 => vec![
            TestIndividual::builder()
                .side(side::Side::A)
                .position(WorldVec3::new(250., 250., 0.))
                .weapons(
                    TestWeapons::builder()
                        .primary(TestWeapon::filled(mod_, "Weapon3").make())
                        .build()
                        .make(),
                )
                .build()
                .make(&w),
            TestIndividual::builder()
                .side(side::Side::B)
                .position(WorldVec3::new(250., 150., 0.))
                .build()
                .make(&w),
            TestIndividual::builder()
                .side(side::Side::B)
                .position(WorldVec3::new(260., 150., 0.))
                .build()
                .make(&w),
        ],
        TestCase::FarMachineGun => vec![
            TestIndividual::builder()
                .side(side::Side::A)
                .position(WorldVec3::new(350., 450., 0.))
                .weapons(
                    TestWeapons::builder()
                        .primary(TestWeapon::filled(mod_, "FullFast").make())
                        .build()
                        .make(),
                )
                .build()
                .make(&w),
            TestIndividual::builder()
                .side(side::Side::B)
                .position(WorldVec3::new(50., 50., 0.))
                .build()
                .make(&w),
        ],
        TestCase::Suppressed => [
            vec![
                TestIndividual::builder()
                    .side(side::Side::A)
                    .position(WorldVec3::new(104., 311., 0.))
                    .weapons(
                        TestWeapons::builder()
                            .primary(TestWeapon::filled(mod_, "Weapon3").make())
                            .build()
                            .make(),
                    )
                    .build()
                    .make(&w),
            ],
            (0..10)
                .map(|i| {
                    TestIndividual::builder()
                        .side(side::Side::B)
                        .position(WorldVec3::new(387., 160. + i as f32 * 10., 0.))
                        .weapons(
                            TestWeapons::builder()
                                .primary(TestWeapon::filled(mod_, "Weapon3").make())
                                .build()
                                .make(),
                        )
                        .build()
                        .make(&w)
                })
                .collect::<Vec<_>>(),
        ]
        .concat(),
    }
}

fn squads(
    _w: &WorldConfig,
    _tiles: &Vec<Tile>,
    individuals: &Vec<oc_individual::Individual>,
    args: &Args,
) -> Vec<oc_individual::squad::Squad> {
    match args.case {
        TestCase::Direct => vec![
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
        TestCase::Direct2 => vec![
            TestSquad::builder()
                .position(individuals.get(0).unwrap().position.into())
                .members(vec![oc_individual::IndividualIndex(0)])
                .orders(vec![])
                .build()
                .make(),
            TestSquad::builder()
                .position(individuals.get(1).unwrap().position.into())
                .members(vec![
                    oc_individual::IndividualIndex(1),
                    oc_individual::IndividualIndex(2),
                ])
                .orders(vec![])
                .build()
                .make(),
        ],
        TestCase::FarMachineGun => vec![
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
        TestCase::Suppressed => [
            vec![
                TestSquad::builder()
                    .position(individuals.get(0).unwrap().position.into())
                    .members(vec![oc_individual::IndividualIndex(0)])
                    .orders(vec![])
                    .build()
                    .make(),
            ],
            (0..10)
                .map(|i| {
                    TestSquad::builder()
                        .position(individuals.get(i + 1).unwrap().position.into())
                        .members(vec![oc_individual::IndividualIndex(i as u64 + 1)])
                        .orders(vec![])
                        .build()
                        .make()
                })
                .collect::<Vec<_>>(),
        ]
        .concat(),
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
        TestCase::Direct | TestCase::Direct2 | TestCase::FarMachineGun | TestCase::Suppressed => {
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
fn tracking(mut state: ResMut<State>, query: Query<(&IndividualIndex, &Status, &Gesture)>) {
    let args = Args::parse();

    static I0_SEEN_PRONE: AtomicBool = AtomicBool::new(false);
    static I1_SEEN_DEAD: AtomicBool = AtomicBool::new(false);
    static I2_SEEN_DEAD: AtomicBool = AtomicBool::new(false);

    let i0_body = query
        .iter()
        .filter_map(|(i, _, gesture)| {
            (i.0 == oc_individual::IndividualIndex(0)).then(|| &gesture.0.body)
        })
        .next();
    let i1_status = query
        .iter()
        .filter_map(|(i, status, _)| (i.0 == oc_individual::IndividualIndex(1)).then(|| &status.0))
        .next();
    let i2_status = query
        .iter()
        .filter_map(|(i, status, _)| (i.0 == oc_individual::IndividualIndex(2)).then(|| &status.0))
        .next();

    if matches!(i0_body, Some(&oc_individual::BodyGesture::Prone(_))) {
        I0_SEEN_PRONE.store(true, Ordering::Relaxed);
    }
    if i1_status == Some(&oc_individual::Status::Dead) {
        I1_SEEN_DEAD.store(true, Ordering::Relaxed);
    }
    if i2_status == Some(&oc_individual::Status::Dead) {
        I2_SEEN_DEAD.store(true, Ordering::Relaxed);
    }

    if match args.case {
        TestCase::Direct | TestCase::FarMachineGun => {
            I1_SEEN_DEAD.load(Ordering::Relaxed) && I0_SEEN_PRONE.load(Ordering::Relaxed)
        }
        TestCase::Direct2 => {
            I1_SEEN_DEAD.load(Ordering::Relaxed)
                && I2_SEEN_DEAD.load(Ordering::Relaxed)
                && I0_SEEN_PRONE.load(Ordering::Relaxed)
        }
        TestCase::Suppressed => false,
    } {
        state.success = Some(Instant::now());
        SUCCESS.store(true, Ordering::Relaxed);
    }
}
