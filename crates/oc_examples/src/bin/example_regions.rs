use std::{
    path::PathBuf,
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};

use bevy::prelude::*;

use anyhow::Context;
use clap::{Parser, ValueEnum};
use oc_battle_gui::{
    ingame::{
        FirstIngameEnter,
        camera::GoToPoint,
        individual::ForgotIndividual,
        input::{individual::InsertIndividualEvent, projectile::InsertProjectileEvent},
        projectile::ForgotProjectile,
    },
    network::output::ToServerEvent,
    states::Game,
};
use oc_examples::{logging, run, snapshot::SnapshotBuilder};
use oc_individual::{IndividualIndex, order::Order};
use oc_mod::Mod;
use oc_network::ToServer;
use oc_projectile::{ProjectileId, spawn::SpawnProjectile};
use oc_root::{
    WorldConfig,
    geo::{WorldVec2, WorldVec3},
    physics::Meters,
};
use oc_world::{meta::Meta, tile::Tile};
use tests::{individual::TestIndividual, squad::TestSquad};

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
impl Args {
    fn timeout(&self) -> Duration {
        match self.case {
            TestCase::ProjectileMoveIn => Duration::from_secs(5),
            TestCase::ProjectileMoveOut => Duration::from_secs(5),
            TestCase::IndividualMoveIn => Duration::from_secs(10),
            TestCase::IndividualMoveOut => Duration::from_secs(10),
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum TestCase {
    ProjectileMoveIn,
    ProjectileMoveOut,
    IndividualMoveIn,
    IndividualMoveOut,
}

fn main() -> Result<(), anyhow::Error> {
    logging::setup_logging()?;

    let args = Args::parse();
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
        .regions_width(2)
        .regions_height(2)
        .region_width(10)
        .region_height(10)
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
        TestCase::ProjectileMoveIn => vec![],
        TestCase::ProjectileMoveOut => vec![],
        TestCase::IndividualMoveIn => vec![
            TestIndividual::builder()
                .position(WorldVec3::new(375., 248., 0.))
                .build()
                .make(&w),
        ],
        TestCase::IndividualMoveOut => vec![
            TestIndividual::builder()
                .position(WorldVec3::new(210., 248., 0.))
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
        TestCase::ProjectileMoveIn => {
            vec![]
        }
        TestCase::ProjectileMoveOut => vec![],
        TestCase::IndividualMoveIn => vec![
            TestSquad::builder()
                .position(individuals.first().unwrap().position.into())
                .members(vec![oc_individual::IndividualIndex(0)])
                .orders(vec![Order::MoveTo(WorldVec2::new(250., 250.))])
                .build()
                .make(),
        ],
        TestCase::IndividualMoveOut => vec![
            TestSquad::builder()
                .position(individuals.first().unwrap().position.into())
                .members(vec![oc_individual::IndividualIndex(0)])
                .orders(vec![Order::MoveTo(WorldVec2::new(10., 250.))])
                .build()
                .make(),
        ],
    }
}

static SUCCESS: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Resource, Default)]
struct State {
    success: Option<Instant>,
    individuals: Vec<IndividualIndex>,
    projectiles: Vec<ProjectileId>,
}

fn install(app: &mut bevy::app::App) {
    let args = Args::parse();

    if args.test {
        app.add_systems(Update, test_tracker);
    }

    app.init_resource::<State>()
        .add_observer(on_first_ingame_enter);

    match args.case {
        TestCase::ProjectileMoveIn => {
            app.add_observer(on_insert_projectile);
        }
        TestCase::ProjectileMoveOut => {
            app.add_observer(on_remove_projectile);
        }
        TestCase::IndividualMoveIn => {
            app.add_observer(on_insert_individual);
        }
        TestCase::IndividualMoveOut => {
            app.add_observer(on_remove_individual);
        }
    }
}

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

fn on_first_ingame_enter(_: On<FirstIngameEnter>, mut commands: Commands) {
    let args = Args::parse();
    let mod_ = Mod::load(&PathBuf::from(MOD), None).unwrap();

    let weapon1 = mod_.weapons.iter().find(|w| w.name() == "Weapon1").unwrap();
    let ammunition = weapon1
        .ammunitions()
        .iter()
        .find(|a| a.name() == "Ammo1")
        .unwrap();
    let shot = weapon1
        .shots()
        .iter()
        .find(|s| s.name() == "Single")
        .unwrap();

    match args.case {
        TestCase::ProjectileMoveIn => {
            let spawn = SpawnProjectile::new(
                weapon1.index(),
                ammunition.index(),
                shot.index(),
                1,
                [450., 248., 8.5].into(),
                [100., 248., 8.5].into(),
            );
            commands.trigger(ToServerEvent(ToServer::SpawnProjectile(spawn)));
        }
        TestCase::ProjectileMoveOut => {
            let spawn = SpawnProjectile::new(
                weapon1.index(),
                ammunition.index(),
                shot.index(),
                1,
                [250., 248., 8.5].into(),
                [100., 248., 8.5].into(),
            );
            commands.trigger(ToServerEvent(ToServer::SpawnProjectile(spawn)));
        }
        TestCase::IndividualMoveIn => {}
        TestCase::IndividualMoveOut => {}
    }

    commands.trigger(GoToPoint(WorldVec2::new(250., 250.)));
}

fn on_insert_projectile(event: On<InsertProjectileEvent>, mut state: ResMut<State>) {
    state.projectiles.push(event.0);
    SUCCESS.store(true, Ordering::Relaxed);
}

fn on_remove_projectile(event: On<ForgotProjectile>, mut state: ResMut<State>) {
    state.projectiles.push(event.0);
    SUCCESS.store(true, Ordering::Relaxed);
}

fn on_insert_individual(event: On<InsertIndividualEvent>, mut state: ResMut<State>) {
    state.individuals.push(event.0);
    SUCCESS.store(true, Ordering::Relaxed);
}

fn on_remove_individual(event: On<ForgotIndividual>, mut state: ResMut<State>) {
    state.individuals.push(event.0);
    SUCCESS.store(true, Ordering::Relaxed);
}
