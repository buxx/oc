use std::path::PathBuf;

use crate::{
    logging, run,
    snapshot::{EmptyGenerator, SnapshotBuilder},
};
use anyhow::Context;
use oc_projectile::spawn::SpawnProjectile;
use oc_root::{WorldConfig, end::End, physics::Meters};
use oc_world::{load::WorldPath, meta::Meta, reader};
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
    let world = Meta::from_file(&map_.meta());
    let world = world.context(format!("Read file {}", map_.meta().display()))?;
    let w = WorldConfig::new(
        map.width().unwrap() as u64,
        map.height().unwrap() as u64,
        Meters(world.geo_meters_per_z),
    );
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
