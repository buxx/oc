use std::path::PathBuf;

use crate::{
    logging, run,
    snapshot::{EmptyGenerator, SnapshotBuilder},
};
use anyhow::Context;
use oc_mod::Mod;
use oc_root::{WorldConfig, physics::Meters};
use oc_world::{load::WorldPath, meta::Meta, reader};
#[cfg(feature = "test")]
use oc_world_server::tracker::Tracker;

#[cfg(feature = "test")]
type Result_ = Result<Tracker, anyhow::Error>;

#[cfg(not(feature = "test"))]
type Result_ = Result<(), anyhow::Error>;

// TODO: its ugly to give directly the projectiles
pub fn run(install: Option<Box<dyn Fn(&mut bevy::app::App)>>) -> Result_ {
    logging::setup_logging().context("Setup logging")?;

    let mod_ = Mod::load(&PathBuf::from("mods/tests1"), None)?;
    let map_ = PathBuf::from("examples/wall");
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

    run::Example::builder()
        .world(map_)
        .mod_(PathBuf::from("mods/tests1"))
        .maybe_install(install)
        .snapshot(snapshot)
        .build()
        .run()
}
