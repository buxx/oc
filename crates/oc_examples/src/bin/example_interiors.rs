#![allow(clippy::unwrap_used)]

use std::path::PathBuf;

use anyhow::Context;
use clap::{Parser, ValueEnum};
use oc_examples::{logging, run, snapshot::SnapshotBuilder};
use oc_mod::Mod;
use oc_root::{WorldConfig, geo::WorldVec3, physics::Meters, side::Side};
use oc_world::{load::WorldPath, meta::Meta, tile::Tile};
use tests::{individual::TestIndividual, squad::TestSquad};

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg()]
    case: Case,

    #[arg(long, action)]
    test: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Case {
    Enter,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    if args.test {
        #[cfg(any(not(feature = "test"), not(feature = "debug")))]
        {
            panic!("To enable test, feature `test` and `debug` must be enabled too")
        }
    }

    logging::setup_logging()?;

    let mod__ = PathBuf::from("mods/tests1");
    let mod_ = Mod::load(&mod__, None)?;
    let map_ = PathBuf::from("examples/minidblue");
    let map = oc_world::reader::MapReader::new(&map_);
    let map = map.context(format!("Read map {}", map_.display()))?;
    let world = Meta::from_file(&map_.meta());
    let world = world.context(format!("Read file {}", map_.meta().display()))?;
    let w = WorldConfig::new(map.width().unwrap() as u64, map.height().unwrap() as u64)
        .with_geo_meters_per_z(Meters(world.geo_meters_per_z));
    let tiles = map.tiles(&w, &mod_).unwrap();

    let individuals = individuals(&w, &mod_, &tiles, &args);
    let squads = squads(&w, &tiles, &individuals, &args);
    let snapshot = SnapshotBuilder::new(map, individuals, squads, vec![]).build(w, &mod_)?;

    let example = run::Example::builder()
        .world(map_)
        .mod_(mod__)
        .snapshot(snapshot)
        .test_app_exit_code(args.test);

    #[cfg(feature = "test")]
    let example = example.install(Box::new(install));

    example.build().run()?;

    if args.test {
        // if !SUCCESS.load(Ordering::Relaxed) {
        //     anyhow::bail!("❌ Test failed")
        // }
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
        Case::Enter => vec![
            TestIndividual::builder()
                .side(Side::A)
                .position(WorldVec3::new(890., 50., 0.)) // FIXME BS NOW: z (auto z in backend ?)
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
        Case::Enter => vec![
            TestSquad::builder()
                .position(individuals.first().unwrap().position.into())
                .members(vec![oc_individual::IndividualIndex(0)])
                .orders(vec![])
                .build()
                .make(),
        ],
    }
}

#[cfg(feature = "test")]
fn install(app: &mut bevy::app::App) {
    let args = Args::parse();

    // #[cfg(feature = "test")]
    // if args.test {
    //     app.add_systems(Update, test_tracker)
    //         .add_systems(Startup, setup)
    //         .add_observer(on_set_status)
    //         .add_observer(on_set_gesture);
    // }

    // #[cfg(feature = "test")]
    // app.init_resource::<State>();

    // match args.case {
    //     TestCase::Direct
    //     | TestCase::Direct2
    //     | TestCase::FarMachineGun
    //     | TestCase::Suppressed
    //     | TestCase::MoveToHiding
    //     | TestCase::MoveToHidingThenDiscover
    //     | TestCase::Hedge => {
    //         #[cfg(feature = "test")]
    //         app.add_systems(Update, (tracking,))
    //             .add_observer(on_spawn_projectile);
    //     }
    // }
}
