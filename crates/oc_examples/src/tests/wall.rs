use std::path::PathBuf;

use crate::{
    logging, run,
    snapshot::{EmptyGenerator, SnapshotBuilder},
};
use anyhow::Context;
use oc_projectile::spawn::SpawnProjectile;
use oc_root::{WorldConfig, end::End};
use oc_world::reader;
#[cfg(feature = "test")]
use oc_world_server::tracker::Tracker;

#[cfg(feature = "test")]
type Result_ = Result<Tracker, Box<dyn std::error::Error>>;

#[cfg(not(feature = "test"))]
type Result_ = Result<(), Box<dyn std::error::Error>>;

// TODO: its ugly to give directly the projectiles
pub fn run(_projectiles: Vec<SpawnProjectile>, _end: End) -> Result_ {
    logging::setup_logging()?;

    let map_ = PathBuf::from("examples/wall");
    let map = reader::MapReader::new(&map_);
    let map = map.context(format!("Read map {}", map_.display()))?;
    let w = WorldConfig::new(map.width().unwrap() as u64, map.height().unwrap() as u64);
    let snapshot =
        SnapshotBuilder::new(map, EmptyGenerator::new(), EmptyGenerator::new()).build(w)?;

    let example = run::Example::builder()
        .world(map_)
        .mod_(PathBuf::from("mods/std1"))
        .snapshot(snapshot);
    #[cfg(feature = "test")]
    let example = example.projectiles(_projectiles).end(_end);
    let result = example.build().run()?;

    Ok(result)
}
