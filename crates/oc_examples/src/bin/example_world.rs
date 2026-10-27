#![allow(clippy::unwrap_used)]

use std::path::PathBuf;
#[cfg(feature = "test")]
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(feature = "test")]
use std::time::Duration;

use anyhow::Context;
use bevy::prelude::*;
use clap::{Parser, ValueEnum};
#[cfg(feature = "test")]
use oc_battle_gui::fx::FxEvent;
use oc_battle_gui::ingame::GameConfigReceived;
use oc_battle_gui::network::output::ToServerEvent;
use oc_examples::{logging, run, snapshot::SnapshotBuilder};
use oc_mod::Mod;
use oc_network::ToServer;
use oc_root::{WorldConfig, battle::BattlePhase, geo::WorldVec2, physics::Meters};
#[cfg(feature = "test")]
use oc_utils::d2::AlmostEqual;
use oc_world::{
    meta::Meta,
    support::{Support, SupportType, Supports},
};

#[cfg(feature = "test")]
static SUCCESS: AtomicBool = AtomicBool::new(false);

const MAP: &str = "examples/meadow1";

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg()]
    case: Case,

    #[arg(long)]
    test: bool,

    #[arg(long = "mod", default_value = "tests1")]
    mod_: String,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Case {
    Support,
}

#[cfg(feature = "test")]
#[derive(Resource)]
struct Timeout(Duration);

#[derive(Resource)]
struct Center(WorldVec2);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    logging::setup_logging()?;

    #[cfg(not(feature = "test"))]
    if args.test {
        panic!("Execute with --test need `test` feature enabled")
    }

    let mod_path = PathBuf::from("mods").join(&args.mod_);
    let mod_ = Mod::load(&mod_path, None)?;
    let meta = Meta::from_file(&PathBuf::from(MAP).join("meta.toml"))?;
    let map = oc_world::reader::MapReader::new(&PathBuf::from(MAP));
    let map = map.context(format!("Read map {MAP}"))?;
    let w = WorldConfig::new(map.width().unwrap() as u64, map.height().unwrap() as u64)
        .with_geo_meters_per_z(Meters(meta.geo_meters_per_z));
    // WorldVec2 is in pixels
    let speed = w.speed();
    let center = WorldVec2::new(
        w.world_width_pixels() as f32 / 2.,
        w.world_height_pixels() as f32 / 2.,
    );

    let (phase, supports) = match args.case {
        Case::Support => (
            BattlePhase::Deployment,
            Supports::new(
                Support(vec![(SupportType::NavalBombard, true)]),
                Support::default(),
            ),
        ),
    };

    let snapshot = SnapshotBuilder::new(map, vec![], vec![], vec![], phase, vec![], vec![])
        .with_supports(supports)
        .build(w, &mod_)?;

    let example = run::Example::builder()
        .world(PathBuf::from(MAP))
        .mod_(mod_path)
        .install(Box::new(move |app| setup(app, center, speed)))
        .snapshot(snapshot);
    example.build().run()?;

    #[cfg(feature = "test")]
    match SUCCESS.load(Ordering::Relaxed) {
        true => println!("✅ SUCCESS !"),
        false => return Err("❌ FAILED !".into()),
    }

    Ok(())
}

fn setup(app: &mut App, center: WorldVec2, #[allow(unused)] speed: f32) {
    let args = Args::parse();

    app.insert_resource(Center(center));

    match args.case {
        Case::Support => {
            app.init_resource::<ConfigReceived>()
                .add_observer(on_game_config_received)
                .add_systems(Update, request_support_and_fight);
        }
    }

    #[cfg(feature = "test")]
    {
        let timeout = Duration::from_secs(30).div_f32(speed);
        app.insert_resource(Timeout(timeout.max(Duration::from_secs(5))))
            .add_observer(on_fx)
            .add_systems(Update, timeout_exit);
    }
}

#[derive(Resource, Default)]
struct ConfigReceived(bool);

fn on_game_config_received(_: On<GameConfigReceived>, mut received: ResMut<ConfigReceived>) {
    received.0 = true;
}

fn request_support_and_fight(
    mut received: ResMut<ConfigReceived>,
    center: Res<Center>,
    mut commands: Commands,
) {
    if !std::mem::take(&mut received.0) {
        return;
    }
    commands.trigger(ToServerEvent(ToServer::RequestSupport(
        SupportType::NavalBombard,
        center.0,
    )));
    commands.trigger(ToServerEvent(ToServer::Fight));
}

#[cfg(feature = "test")]
fn on_fx(fx: On<FxEvent>, center: Res<Center>, mut commands: Commands) {
    use oc_physics::fx::{Fx, Visual};

    let Fx::Visual(Visual::Explosion(position, _)) = &fx.0 else {
        return;
    };

    if position.almost_equal(center.0, 10.) {
        SUCCESS.store(true, Ordering::Relaxed);
        commands.write_message(AppExit::from_code(0));
    }
}

#[cfg(feature = "test")]
fn timeout_exit(time: Res<Time>, timeout: Res<Timeout>, mut commands: Commands) {
    if time.elapsed() > timeout.0 {
        commands.write_message(AppExit::from_code(0));
    }
}
