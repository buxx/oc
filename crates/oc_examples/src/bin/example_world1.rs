use std::path::PathBuf;

use anyhow::Context;
use oc_examples::{logging, run, snapshot::SnapshotBuilder};
use oc_mod::Mod;
use oc_root::{WorldConfig, physics::Meters};
use oc_world::meta::Meta;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    logging::setup_logging()?;

    let mod_ = Mod::load(&PathBuf::from("mods/std1"), None)?;
    let meta = Meta::from_file(&PathBuf::from("examples/world1/meta.toml"))?;
    let map_ = PathBuf::from("examples/world1");
    let map = oc_world::reader::MapReader::new(&map_);
    let map = map.context(format!("Read map {}", map_.display()))?;
    let w = WorldConfig::new(
        map.width().unwrap() as u64,
        map.height().unwrap() as u64,
        Meters(meta.geo_meters_per_z),
    );
    let snapshot = SnapshotBuilder::new(map, vec![], vec![], vec![]).build(w, &mod_)?;

    let example = run::Example::builder()
        .world(PathBuf::from("examples/world1"))
        .mod_(PathBuf::from("mods/std1"))
        .snapshot(snapshot);
    example.build().run()?;

    Ok(())
}
