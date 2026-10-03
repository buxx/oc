#![allow(clippy::unwrap_used)]

#[cfg(feature = "test")]
use std::time::Instant;
use std::{path::PathBuf, time::Duration};

use anyhow::Context;
use bevy::prelude::*;
use clap::{Parser, ValueEnum};
use oc_battle_gui::ingame::{GameConfigReceived, camera::move_::CenterCameraOn};
#[cfg(feature = "test")]
use oc_battle_gui::states::GameConfig;
use oc_examples::{logging, run, snapshot::SnapshotBuilder};
use oc_geo::tile::WorldTileIndex;
use oc_individual::order::Order;
use oc_mod::Mod;
use oc_root::{
    WcfgInto, WorldConfig, geo::WorldVec2, physics::Meters, side::Side, utils::Frequency,
};
#[cfg(feature = "test")]
use oc_world::interior::Interior;
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
    Out,
}

fn main() -> Result<(), anyhow::Error> {
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
    let speed = w.speed();
    let w = w.with_interiors_tick(Frequency::each(Duration::from_secs(1).div_f32(speed)));
    let tiles = map.tiles(&w, &mod_).unwrap();

    let individuals = individuals(&w, &mod_, &tiles, &args);
    let squads = squads(&w, &tiles, &individuals, &args);
    let snapshot = SnapshotBuilder::new(map, individuals, squads, vec![]).build(w, &mod_)?;

    let example = run::Example::builder()
        .world(map_)
        .mod_(mod__)
        .snapshot(snapshot)
        .test_app_exit_code(args.test);

    let example = example.install(Box::new(install));

    example.build().run()?;

    Ok(())
}

fn individuals(
    w: &WorldConfig,
    _mod_: &oc_mod::Mod,
    tiles: &Vec<Tile>,
    args: &Args,
) -> Vec<oc_individual::Individual> {
    match args.case {
        Case::Enter => {
            let position = WorldVec2::new(915., 50.);
            let tile: WorldTileIndex = position.into_(&w);
            let tile = &tiles[tile.0 as usize];
            let position = position.extend(tile.z_pixels(w));
            vec![
                TestIndividual::builder()
                    .side(Side::A)
                    .position(position)
                    .build()
                    .make(w),
            ]
        }
        Case::Out => {
            let position = WorldVec2::new(850., 70.);
            let tile: WorldTileIndex = position.into_(&w);
            let tile = &tiles[tile.0 as usize];
            let position = position.extend(tile.z_pixels(w));
            vec![
                TestIndividual::builder()
                    .side(Side::A)
                    .position(position)
                    .build()
                    .make(w),
            ]
        }
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
                .orders(vec![Order::MoveFastTo(WorldVec2::new(850., 70.))])
                .build()
                .make(),
        ],
        Case::Out => vec![
            TestSquad::builder()
                .position(individuals.first().unwrap().position.into())
                .members(vec![oc_individual::IndividualIndex(0)])
                .orders(vec![Order::MoveFastTo(WorldVec2::new(915., 50.))])
                .build()
                .make(),
        ],
    }
}

fn install(app: &mut bevy::app::App) {
    let args = Args::parse();

    match args.case {
        Case::Enter | Case::Out => {
            // Center screen on the house
            app.add_observer(|_: On<GameConfigReceived>, mut commands: Commands| {
                commands.trigger(CenterCameraOn(WorldVec2::new(885., 50.)));
            });
        }
    }

    #[cfg(feature = "test")]
    if args.test {
        let state = State {
            start: Instant::now(),
        };
        app.insert_resource(state).add_systems(Update, tracking);
    }
}

#[cfg(feature = "test")]
#[derive(Debug, Resource)]
struct State {
    start: Instant,
}

#[cfg(feature = "test")]
fn tracking(
    mut commands: Commands,
    g: Res<GameConfig>,
    query: Query<(&Interior, &Visibility)>,
    state: Res<State>,
) {
    let_some!(g = &g.0, return);
    let speed = g.w.speed();
    let args = Args::parse();

    let timeout = match args.case {
        Case::Enter | Case::Out => Duration::from_secs(10),
    }
    .div_f32(speed)
    .max(Duration::from_secs(2));

    if {
        match args.case {
            Case::Enter => query.iter().any(|(_, v)| matches!(v, Visibility::Visible)),
            Case::Out => {
                state.start.elapsed().as_secs_f32() > 2. / speed
                    && query.iter().any(|(_, v)| matches!(v, Visibility::Hidden))
            }
        }
    } {
        println!("✅ Success !");
        commands.write_message(bevy::app::AppExit::from_code(0));
    }

    if state.start.elapsed() > timeout {
        println!("❌ Test failed !");
        commands.write_message(bevy::app::AppExit::from_code(1));
    }
}
