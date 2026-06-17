use std::{
    path::PathBuf,
    sync::Mutex,
    time::{Duration, Instant},
};

use anyhow::Context;
use bevy::prelude::*;
use clap::Parser;
use oc_battle_gui::{
    ingame::{FirstIngameEnter, individual::Status},
    network::output::ToServerEvent,
    states::Game,
};
use oc_examples::{logging, run, snapshot::SnapshotBuilder};
use oc_geo::{
    region::WorldRegionIndex,
    tile::{TileXy, WorldTileIndex},
};
use oc_individual::{IndividualIndex, order::Order};
use oc_mod::Mod;
use oc_network::ToServer;
use oc_projectile::spawn::SpawnProjectile;
use oc_root::{WcfgFrom, WorldConfig, physics::Meters};
use oc_utils::d2::{Direction, Xy};
use oc_world::{meta::Meta, tile::Tile};

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long, action)]
    test: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    logging::setup_logging()?;

    let args = Args::parse();
    if args.test {
        #[cfg(not(feature = "test"))]
        {
            panic!("To enable test, feature `test` must be enabled too")
        }
    }

    let mod_ = PathBuf::from("mods/tests1");
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
    let (individuals, squads) = individuals(&w, &tiles);
    let snapshot = SnapshotBuilder::new(map_, individuals, squads, vec![]).build(w, &mod__)?;

    let example = run::Example::builder()
        .world(map)
        .mod_(mod_)
        .install(Box::new(install))
        .test_app_exit_code(args.test)
        .snapshot(snapshot);

    #[allow(unused)]
    let tracker = example.build().run()?;

    if args.test {
        #[cfg(feature = "test")]
        {
            use oc_world_server::state::ObjectId;

            let tracker = tracker.take();

            // We consider success if physics event own at leat 10 projectiles collisions
            let collision = tracker.physics.iter().find(|event| {
                matches!(
                    event,
                    oc_physics::Event::Collision(ObjectId::Projectile(_), ObjectId::Individual(_))
                )
            });
            let dead = tracker.individuals.iter().find(|event| {
                matches!(
                    event,
                    (
                        IndividualIndex(0),
                        oc_individual::Update::SetStatus(oc_individual::Status::Dead)
                    )
                )
            });

            assert!(collision.is_some());
            assert!(dead.is_some());

            println!("✅ (SERVER) All tests passed");
        }
    }

    Ok(())
}

fn individuals(
    _: &WorldConfig,
    _: &Vec<Tile>,
) -> (
    Vec<oc_individual::Individual>,
    Vec<oc_individual::squad::Squad>,
) {
    let positions = vec![[150.0, 150.0, 0.0]];

    // TODO: avoid repetition with main()
    let meta = Meta::from_file(&PathBuf::from("examples/meadow1/meta.toml")).unwrap();
    let map_ = PathBuf::from("examples/meadow1");
    let map = oc_world::reader::MapReader::new(&map_).unwrap();
    let w = WorldConfig::new(
        map.width().unwrap() as u64,
        map.height().unwrap() as u64,
        Meters(meta.geo_meters_per_z),
    );

    let individuals = positions
        .iter()
        .map(|p| {
            let tile_xy = TileXy(Xy(
                p[0] as u64 / w.geo_pixels_per_tile,
                p[1] as u64 / w.geo_pixels_per_tile,
            ));
            let tile = WorldTileIndex::from_(tile_xy, &w);

            oc_individual::Individual::new(
                p.clone(),
                tile,
                WorldRegionIndex(0),
                vec![],
                oc_individual::behavior::Behavior::Idle(Direction::NORTH),
                vec![],
                oc_individual::Status::Operational,
                oc_individual::Gesture::Idle(Direction::NORTH),
                oc_individual::behavior::Intent::Idle(Direction::NORTH),
            )
        })
        .collect();

    let squads = positions
        .iter()
        .enumerate()
        .map(|(i, _)| {
            let individual = IndividualIndex(i as u64);
            oc_individual::squad::Squad {
                members: vec![individual],
                orders: vec![Order::Idle],
            }
        })
        .collect();

    (individuals, squads)
}

fn install(app: &mut bevy::app::App) {
    let args = Args::parse();

    if args.test {
        app.add_systems(
            Update,
            |mut commands: Commands,
             game: Res<Game>,
             individuals: Query<
                &Status,
                With<oc_battle_gui::entity::individual::IndividualIndex>,
            >| {
                static DEAD: Mutex<Option<Instant>> = Mutex::new(None);

                let timeout = game.started.elapsed() > Duration::from_secs(10);
                let mut dead = DEAD.lock().unwrap();
                *dead = match *dead {
                    None => individuals
                        .iter()
                        .find(|status| matches!(status.0, oc_individual::Status::Dead))
                        .is_some()
                        .then(|| Instant::now()),
                    Some(value) => {
                        if value.elapsed().as_secs() > 1 {
                            println!("✅ (GUI) Individual is dead");
                            commands.write_message(bevy::app::AppExit::from_code(0));
                        };
                        Some(value)
                    }
                };
                if timeout {
                    eprintln!("❌ (GUI) Timeout reached ! Individual is not dead.");
                    commands.write_message(bevy::app::AppExit::from_code(1));
                }
            },
        );
    }

    app.add_observer(on_first_ingame_enter);
}

fn on_first_ingame_enter(_: On<FirstIngameEnter>, mut commands: Commands) {
    let mod_ = Mod::load(&PathBuf::from("mods/tests1"), None).unwrap();

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

    for (start, end) in vec![([220.0, 150.0, 5.0], [100.0, 150.0, 5.0])] {
        commands.trigger(ToServerEvent(ToServer::SpawnProjectile(
            SpawnProjectile::new(
                weapon1.index(),
                ammunition.index(),
                shot.index(),
                1,
                start,
                end,
            ),
        )));
    }
}
