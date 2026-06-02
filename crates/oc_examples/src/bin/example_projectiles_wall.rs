use std::{path::PathBuf, time::Duration};

use bevy::prelude::*;
use clap::Parser;
use oc_battle_gui::{ingame::FirstIngameEnter, network::output::ToServerEvent, states::Game};
use oc_mod::Mod;
use oc_network::ToServer;

use {oc_examples::tests::wall, oc_projectile::spawn::SpawnProjectile};

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long, action)]
    test: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    if args.test {
        #[cfg(not(feature = "test"))]
        {
            panic!("To enable test, feature `test` must be enabled too")
        }
    }

    #[allow(unused)]
    let tracker = wall::run(Some(Box::new(install))).unwrap();

    if args.test {
        #[cfg(feature = "test")]
        {
            use oc_world_server::state::ObjectId;

            let tracker = tracker.take();
            // We consider success if physics event own at leat 10 projectiles collisions
            let collisions = tracker
                .physics
                .iter()
                .filter(|e| match e {
                    oc_physics::Event::Collision(ObjectId::Projectile(_), ObjectId::Tile(_)) => {
                        true
                    }
                    _ => false,
                })
                .count();

            assert!(collisions >= 10);
            println!("✅ Test success");
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

    for spawn in [
        SpawnProjectile::new(
            weapon1.index(),
            ammunition.index(),
            shot.index(),
            10,
            [0., 0., 15.],
            [50., 50., 15.],
        ),
        SpawnProjectile::new(
            weapon2.index(),
            ammunition.index(),
            shot.index(),
            10,
            [10., 0., 15.],
            [60., 50., 15.],
        ),
        SpawnProjectile::new(
            weapon3.index(),
            ammunition.index(),
            shot3.index(),
            10,
            [20., 0., 15.],
            [70., 50., 15.],
        ),
    ] {
        commands.trigger(ToServerEvent(ToServer::SpawnProjectile(spawn)));
    }
}
