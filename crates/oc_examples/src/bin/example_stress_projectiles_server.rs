use std::{
    path::PathBuf,
    sync::{Arc, mpsc::channel},
    time::Duration,
};

use anyhow::Context;
use bevy::prelude::*;
use oc_examples::{logging, snapshot::SnapshotBuilder};
use oc_mod::Mod;
use oc_network::ToServer;
use oc_projectile::spawn::SpawnProjectile;
use oc_root::{WorldConfig, physics::Meters, static_::StaticSource};
use oc_world::meta::Meta;
use oc_world_server::{bridge::Event, config::ServerConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    logging::setup_logging()?;

    let cache = PathBuf::from(".cache");
    let world = PathBuf::from("examples/world1");
    let mod_ = PathBuf::from("mods/tests1");
    let mod__ = Mod::load(&mod_, None)?;
    let static_ = StaticSource::Remote(32000);
    let meta = Meta::from_file(&PathBuf::from("examples/world1/meta.toml"))?;
    let map_ = PathBuf::from("examples/world1");
    let map = oc_world::reader::MapReader::new(&map_);
    let map = map.context(format!("Read map {}", map_.display()))?;
    let w = WorldConfig::new(
        map.width().unwrap() as u64,
        map.height().unwrap() as u64,
        Meters(meta.geo_meters_per_z),
    );
    let snapshot = SnapshotBuilder::new(map.clone(), vec![], vec![]).build(w.clone(), &mod__)?;
    let config = ServerConfig::builder()
        .world(world.clone())
        .mod_(mod_.clone())
        .cache(cache)
        .static_(static_)
        .snapshot(snapshot.clone())
        .build();
    let state = oc_world_server::state::init::<()>(config.clone())?;
    let state = Arc::new(state);

    let (ready_tx, ready_rx) = channel::<std::result::Result<(), String>>();
    let (to_client_tx, _) = channel();
    let (to_server_tx, to_server_rx) = channel();

    std::thread::spawn(move || {
        oc_world_server::runner::Runner::new(
            config,
            state,
            to_client_tx,
            #[cfg(feature = "test")]
            tracker,
        )
        .run(to_server_rx, ready_tx);
    });

    let _ = ready_rx.recv().unwrap();

    let weapon = mod__
        .weapons
        .iter()
        .find(|w| w.name() == "Weapon1")
        .unwrap();
    let ammunition = weapon
        .ammunitions()
        .iter()
        .find(|a| a.name() == "Ammo1")
        .unwrap();
    let shot = weapon
        .shots()
        .iter()
        .find(|s| s.name() == "Single")
        .unwrap();

    let right = map.width().unwrap() as f32 * w.geo_pixels_per_tile as f32;
    let bottom = map.height().unwrap() as f32 * w.geo_pixels_per_tile as f32;

    let mut counter = 0;
    loop {
        if counter < 100_000 {
            for _ in 0..100 {
                to_server_tx
                    .send(Event::Message(
                        (),
                        ToServer::SpawnProjectile(SpawnProjectile::new(
                            weapon.index(),
                            ammunition.index(),
                            shot.index(),
                            1,
                            [0., 0., 500.],
                            [right, bottom, 500.],
                        )),
                    ))
                    .unwrap()
            }
            counter += 1000;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}
