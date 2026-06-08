use std::path::PathBuf;

use anyhow::Context;
use oc_examples::{logging, run, snapshot::SnapshotBuilder};
use oc_mod::Mod;
use oc_root::{WorldConfig, physics::Meters};
use oc_world::{load::WorldPath, meta::Meta};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    logging::setup_logging()?;

    let mod_ = Mod::load(&PathBuf::from("mods/std1"), None)?;
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
    let snapshot = SnapshotBuilder::new(map, vec![], vec![], vec![]).build(w, &mod_)?;

    let example = run::Example::builder()
        .world(map_)
        .mod_(PathBuf::from("mods/std1"))
        .snapshot(snapshot);
    example.build().run()?;

    Ok(())
}
