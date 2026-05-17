use std::path::PathBuf;

use anyhow::Context;
use oc_examples::{
    logging, run,
    snapshot::{EmptyGenerator, SnapshotBuilder},
};
use oc_individual::Individual;
use oc_projectile::Projectile;
use oc_root::{WorldConfig, physics::Meters};
use oc_world::{load::WorldPath, meta::Meta, tile::Tile};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    logging::setup_logging()?;

    let map_ = PathBuf::from("examples/minidblue");
    let map = oc_world::reader::MapReader::new(&map_);
    let map = map.context(format!("Read map {}", map_.display()))?;
    let world = Meta::from_file(&map_.meta());
    let world = world.context(format!("Read file {}", map_.meta().display()))?;
    let w = WorldConfig::new(
        map.width().unwrap() as u64,
        map.height().unwrap() as u64,
        Meters(world.geo_meters_per_z),
    );
    let projectiles = EmptyGenerator::<Projectile>::new();
    let snapshot = SnapshotBuilder::new(map, individuals, projectiles).build(w)?;

    let example = run::Example::builder()
        .world(map_)
        .mod_(PathBuf::from("mods/std1"))
        .snapshot(snapshot);
    #[cfg(feature = "test")]
    let example = example.projectiles(vec![]);
    let _ = example.build().run()?;

    Ok(())
}

fn individuals(_w: &WorldConfig, _tiles: &Vec<Tile>) -> Vec<Individual> {
    vec![]
}
