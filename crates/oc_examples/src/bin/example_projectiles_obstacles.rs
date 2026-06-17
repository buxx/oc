use std::{path::PathBuf, time::Duration};

use anyhow::Context;
use bevy::prelude::*;
use clap::{Parser, ValueEnum};
use oc_battle_gui::{ingame::FirstIngameEnter, network::output::ToServerEvent, states::Game};
use oc_examples::{run, snapshot::SnapshotBuilder};
use oc_mod::Mod;
use oc_network::ToServer;
use oc_root::{WorldConfig, physics::Meters};
use oc_world::{load::WorldPath, meta::Meta, reader};
#[cfg(feature = "test")]
use oc_world_server::state::ObjectId;

use oc_projectile::spawn::SpawnProjectile;

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg()]
    case: TestCase,

    #[arg(long, action)]
    test: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum TestCase {
    OneAgainstWall,
    MultipleAgainstWall,
    OneAgainstHill,
    MultipleAgainstHill,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    if args.test {
        #[cfg(not(feature = "test"))]
        {
            panic!("To enable test, feature `test` must be enabled too")
        }
    }

    let mod_ = Mod::load(&PathBuf::from("mods/tests1"), None)?;
    let map_ = PathBuf::from("examples/height");
    let map = reader::MapReader::new(&map_);
    let map = map.context(format!("Read map {}", map_.display()))?;
    let world = Meta::from_file(&map_.meta());
    let world = world.context(format!("Read file {}", map_.meta().display()))?;
    let w = WorldConfig::new(
        map.width().unwrap() as u64,
        map.height().unwrap() as u64,
        Meters(world.geo_meters_per_z),
    );
    let snapshot = SnapshotBuilder::new(map, vec![], vec![], vec![]).build(w, &mod_)?;

    #[allow(unused)]
    let tracker = run::Example::builder()
        .world(map_)
        .mod_(PathBuf::from("mods/tests1"))
        .install(Box::new(install))
        .snapshot(snapshot)
        .test_app_exit_code(args.test)
        .build()
        .run()
        .unwrap();

    if args.test {
        #[cfg(feature = "test")]
        {
            let tracker = tracker.take();

            match args.case {
                TestCase::MultipleAgainstWall | TestCase::MultipleAgainstHill => {
                    // We consider success if physics event own at leat 10 projectiles collisions
                    let collisions = tracker
                        .physics
                        .iter()
                        .filter(|e| match e {
                            oc_physics::Event::Collision(
                                ObjectId::Projectile(_),
                                ObjectId::Tile(_),
                            ) => true,
                            _ => false,
                        })
                        .count();

                    assert!(collisions >= 10);
                    println!("✅ Test success");
                }
                TestCase::OneAgainstWall | TestCase::OneAgainstHill => {
                    // We consider success if physics event own at leat 10 projectiles collisions
                    let collisions = tracker
                        .physics
                        .iter()
                        .filter(|e| match e {
                            oc_physics::Event::Collision(
                                ObjectId::Projectile(_),
                                ObjectId::Tile(_),
                            ) => true,
                            _ => false,
                        })
                        .count();

                    assert!(collisions == 1);
                    println!("✅ Test success");
                }
            };
        }
    }

    Ok(())
}

fn install(app: &mut bevy::app::App) {
    let args = Args::parse();

    if args.test {
        app.add_systems(Update, |mut commands: Commands, game: Res<Game>| {
            if game.started.elapsed() > Duration::from_secs(3) {
                commands.write_message(bevy::app::AppExit::from_code(0));
            }
        });
    }

    app.add_observer(on_first_ingame_enter);
}

fn on_first_ingame_enter(_: On<FirstIngameEnter>, mut commands: Commands) {
    let args = Args::parse();
    let mod_ = Mod::load(&PathBuf::from("mods/tests1"), None).unwrap();

    let weapon1 = mod_.weapons.iter().find(|w| w.name() == "Weapon1").unwrap();
    let weapon2 = mod_.weapons.iter().find(|w| w.name() == "Weapon2").unwrap();
    let weapon3 = mod_.weapons.iter().find(|w| w.name() == "Weapon3").unwrap();
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
    let shot3 = weapon3
        .shots()
        .iter()
        .find(|s| s.name() == "Burst3")
        .unwrap();

    match args.case {
        TestCase::MultipleAgainstHill => {
            for spawn in [
                SpawnProjectile::new(
                    weapon1.index(),
                    ammunition.index(),
                    shot.index(),
                    10,
                    [70., 60., 8.5],
                    [100., 60., 8.5],
                ),
                SpawnProjectile::new(
                    weapon2.index(),
                    ammunition.index(),
                    shot.index(),
                    10,
                    [70., 65., 8.5],
                    [100., 65., 8.5],
                ),
                SpawnProjectile::new(
                    weapon3.index(),
                    ammunition.index(),
                    shot3.index(),
                    10,
                    [70., 70., 8.5],
                    [100., 70., 8.5],
                ),
            ] {
                commands.trigger(ToServerEvent(ToServer::SpawnProjectile(spawn)));
            }
        }
        TestCase::OneAgainstHill => {
            for spawn in [SpawnProjectile::new(
                weapon1.index(),
                ammunition.index(),
                shot.index(),
                1,
                [70., 60., 8.5],
                [100., 60., 8.5],
            )] {
                commands.trigger(ToServerEvent(ToServer::SpawnProjectile(spawn)));
            }
        }
        TestCase::MultipleAgainstWall => {
            for spawn in [
                SpawnProjectile::new(
                    weapon1.index(),
                    ammunition.index(),
                    shot.index(),
                    10,
                    [70., 125., 8.5],
                    [100., 125., 8.5],
                ),
                SpawnProjectile::new(
                    weapon2.index(),
                    ammunition.index(),
                    shot.index(),
                    10,
                    [70., 130., 8.5],
                    [100., 130., 8.5],
                ),
                SpawnProjectile::new(
                    weapon3.index(),
                    ammunition.index(),
                    shot3.index(),
                    10,
                    [70., 135., 8.5],
                    [100., 135., 8.5],
                ),
            ] {
                commands.trigger(ToServerEvent(ToServer::SpawnProjectile(spawn)));
            }
        }
        TestCase::OneAgainstWall => {
            for spawn in [SpawnProjectile::new(
                weapon1.index(),
                ammunition.index(),
                shot.index(),
                1,
                [70., 125., 8.5],
                [100., 125., 8.5],
            )] {
                commands.trigger(ToServerEvent(ToServer::SpawnProjectile(spawn)));
            }
        }
    }
}
