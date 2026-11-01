#![allow(clippy::unwrap_used)]

#[cfg(feature = "test")]
use std::time::Instant;
use std::{path::PathBuf, time::Duration};

use anyhow::Context as ContextExt;
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
    WcfgInto, WorldConfig, battle::BattlePhase, geo::WorldVec2, physics::Meters, side::Side,
    utils::Frequency,
};
#[cfg(feature = "test")]
use oc_utils::let_some;
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
    /// Individual (our side) enter an interior show this interior
    Enter,
    /// Individual (our side) exit an interior hide this interior
    Out,
    /// Individual inside a protected interior survives an explosion outside it
    OutsideExplosionProtected,
    /// Individual inside a protected interior survives a non penetrating explosion inside it
    OnTheTopExplosionProtected,
    /// Individual inside an interior without protection dies of an explosion inside it
    OnTheTopExplosionNotProtected,
    /// Same as `OutsideExplosionProtected`, with an unprotected interior
    OutsideExplosionProtected2,
}

impl Case {
    fn is_explosion(&self) -> bool {
        matches!(
            self,
            Case::OutsideExplosionProtected
                | Case::OnTheTopExplosionProtected
                | Case::OnTheTopExplosionNotProtected
                | Case::OutsideExplosionProtected2
        )
    }

    fn individual_position(&self) -> [f32; 2] {
        match self {
            Case::OnTheTopExplosionNotProtected | Case::OutsideExplosionProtected2 => [111., 192.],
            _ => [185., 344.],
        }
    }

    fn explosion_position(&self) -> [f32; 2] {
        match self {
            Case::OutsideExplosionProtected => [255., 356.],
            Case::OnTheTopExplosionProtected => [180., 341.],
            Case::OnTheTopExplosionNotProtected => [111., 192.],
            Case::OutsideExplosionProtected2 => [118., 211.],
            Case::Enter | Case::Out => unreachable!(),
        }
    }

    fn expect_dead(&self) -> bool {
        matches!(self, Case::OnTheTopExplosionNotProtected)
    }
}

/// "Kill & Stun only (no shrapnel & 0.5 penetration)"
const EXPLOSION_PROTECTED_SHELL_INDEX: oc_mod::shell::ShellIndex = oc_mod::shell::ShellIndex(2);

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
    let snapshot = SnapshotBuilder::new(
        map,
        individuals,
        squads,
        vec![],
        BattlePhase::Fight,
        vec![],
        vec![],
    )
    .build(w, &mod_)?;

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
    tiles: &[Tile],
    args: &Args,
) -> Vec<oc_individual::Individual> {
    match args.case {
        Case::Enter => {
            let position = WorldVec2::new(915., 50.);
            let tile: WorldTileIndex = position.into_(w);
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
            let tile: WorldTileIndex = position.into_(w);
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
        Case::OutsideExplosionProtected
        | Case::OnTheTopExplosionProtected
        | Case::OnTheTopExplosionNotProtected
        | Case::OutsideExplosionProtected2 => {
            let position = WorldVec2::from(args.case.individual_position());
            let tile: WorldTileIndex = position.into_(w);
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
    _tiles: &[Tile],
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
        Case::OutsideExplosionProtected
        | Case::OnTheTopExplosionProtected
        | Case::OnTheTopExplosionNotProtected
        | Case::OutsideExplosionProtected2 => vec![
            TestSquad::builder()
                .position(individuals.first().unwrap().position.into())
                .members(vec![oc_individual::IndividualIndex(0)])
                .build()
                .make(),
        ],
    }
}

fn install(app: &mut bevy::app::App) {
    let args = Args::parse();

    match args.case {
        Case::Enter
        | Case::Out
        | Case::OutsideExplosionProtected
        | Case::OnTheTopExplosionProtected
        | Case::OnTheTopExplosionNotProtected
        | Case::OutsideExplosionProtected2 => {
            // Center screen on the house
            app.add_observer(|_: On<GameConfigReceived>, mut commands: Commands| {
                commands.trigger(CenterCameraOn(WorldVec2::new(885., 50.)));
            });
        }
    }

    if args.case.is_explosion() {
        app.add_observer(|_: On<GameConfigReceived>, mut commands: Commands| {
            commands.trigger(CenterCameraOn(WorldVec2::new(220., 350.)));
        });

        #[cfg(feature = "debug")]
        app.add_systems(Update, fire_explosion);

        #[cfg(feature = "test")]
        if args.test {
            app.init_resource::<ExplosionProtectedTest>()
                .add_systems(Update, explosion_protected_test);
        }

        return;
    }

    #[cfg(feature = "test")]
    if args.test {
        let state = State {
            start: Instant::now(),
        };
        app.insert_resource(state)
            .add_systems(Update, test_interior_visibility);
    }
}

#[cfg(feature = "debug")]
fn fire_explosion(
    mut commands: Commands,
    mut fired: Local<bool>,
    individuals: Query<&oc_battle_gui::entity::individual::IndividualIndex>,
) {
    use oc_battle_gui::network::output::ToServerEvent;
    use oc_network::ToServer;

    // Do not fire twice
    if *fired
    // Do not fire if individual is not yet displayed
        || !individuals
            .iter()
            .any(|i| i.0 == oc_individual::IndividualIndex(0))
    {
        return;
    }

    *fired = true;
    commands.trigger(ToServerEvent(ToServer::DebugExplosion(
        EXPLOSION_PROTECTED_SHELL_INDEX,
        WorldVec2::from(Args::parse().case.explosion_position()),
    )));
}

#[cfg(feature = "test")]
#[derive(Debug, Resource, Default)]
struct ExplosionProtectedTest {
    fired: Option<Instant>,
}

/// Individual must be dead (or not, according to the case) after explosion.
#[cfg(feature = "test")]
fn explosion_protected_test(
    mut commands: Commands,
    mut test: ResMut<ExplosionProtectedTest>,
    g: Res<GameConfig>,
    individuals: Query<(
        &oc_battle_gui::entity::individual::IndividualIndex,
        &oc_battle_gui::ingame::individual::Status,
    )>,
) {
    let_some!(g = &g.0, return);
    let Some((_, status)) = individuals
        .iter()
        .find(|(i, _)| i.0 == oc_individual::IndividualIndex(0))
    else {
        return;
    };
    let fired = *test.fired.get_or_insert_with(Instant::now);

    let dead = matches!(status.0, oc_individual::Status::Dead);
    let elapsed = fired.elapsed() > Duration::from_secs(5).div_f32(g.w.speed());
    if !(dead || elapsed) {
        return;
    }

    if dead == Args::parse().case.expect_dead() {
        println!("✅ Success !");
        commands.write_message(bevy::app::AppExit::from_code(0));
    } else {
        eprintln!("❌ Test failed ! Individual dead: {dead}");
        commands.write_message(bevy::app::AppExit::from_code(1));
    }
}

#[cfg(feature = "test")]
#[derive(Debug, Resource)]
struct State {
    start: Instant,
}

#[cfg(feature = "test")]
fn test_interior_visibility(
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
        _ => return,
    }
    .div_f32(speed)
    .max(Duration::from_secs(2));

    if match args.case {
        Case::Enter => query.iter().any(|(_, v)| matches!(v, Visibility::Visible)),
        Case::Out => {
            state.start.elapsed().as_secs_f32() > 2. / speed
                && query.iter().any(|(_, v)| matches!(v, Visibility::Hidden))
        }
        _ => false,
    } {
        println!("✅ Success !");
        commands.write_message(bevy::app::AppExit::from_code(0));
    }

    if state.start.elapsed() > timeout {
        println!("❌ Test failed !");
        commands.write_message(bevy::app::AppExit::from_code(1));
    }
}
