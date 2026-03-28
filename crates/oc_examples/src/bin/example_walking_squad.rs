use std::path::PathBuf;

use oc_examples::{logging, run};
use oc_root::{REGION_HEIGHT, REGION_WIDTH, WORLD_HEIGHT, WORLD_WIDTH};

// FIXME BS NOW: here, only this example specific. Le reste, dans du code generique de oc_example
fn main() -> Result<(), Box<dyn std::error::Error>> {
    if WORLD_WIDTH != 1000 || WORLD_HEIGHT != 1000 || REGION_WIDTH != 100 || REGION_HEIGHT != 100 {
        panic!("Examples must be started from ./examples.sh script");
    }

    logging::setup_logging()?;

    run::Example::builder()
        .world(PathBuf::from("examples/world1"))
        .mod_(PathBuf::from("mods/std1"))
        .build()
        .run()?;

    Ok(())
}
