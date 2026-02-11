use oc_individual::{Individual, behavior::Behavior};
use oc_root::TILES_COUNT;
use oc_utils::d2::{Xy, XyIndex};
use oc_world::{World, tile::Tile};

use crate::runner::Runner;

mod individual;
mod perf;
mod runner;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tiles = [Tile::ShortGrass; TILES_COUNT];
    let individuals =
        std::array::from_fn(|i| Individual::new(Xy::from(XyIndex(i)), Behavior::MovingSouth));
    let world = World::new(tiles, individuals);

    Runner::new(world).run()?;
    Ok(())
}
