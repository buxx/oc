use std::path::PathBuf;

use anyhow::Context;
use oc_examples::{
    logging, run,
    snapshot::{EmptyGenerator, SnapshotBuilder},
};
use oc_root::{REGION_HEIGHT, REGION_WIDTH, WORLD_HEIGHT, WORLD_WIDTH};
use oc_world::reader;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Find a way to automatize/standadize that
    if WORLD_WIDTH != 10 || WORLD_HEIGHT != 10 || REGION_WIDTH != 10 || REGION_HEIGHT != 10 {
        panic!("Examples must be started from ./examples.sh script");
    }

    logging::setup_logging()?;

    let map_ = PathBuf::from("examples/wall");
    let map = reader::MapReader::new(&map_);
    let map = map.context(format!("Read map {}", map_.display()))?;
    let snapshot =
        SnapshotBuilder::new(map, EmptyGenerator::new(), EmptyGenerator::new()).build()?;

    run::Example::builder()
        .world(map_)
        .mod_(PathBuf::from("mods/std1"))
        .snapshot(snapshot)
        .build()
        .run()?;

    Ok(())
}
