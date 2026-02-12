use oc_individual::{Individual, behavior::Behavior};
use oc_root::TILES_COUNT;
use oc_utils::d2::{Xy, XyIndex};
use oc_world::{World, tile::Tile};

use crate::{network::Network, runner::Runner, state::State};

mod index;
mod individual;
mod network;
mod perf;
mod runner;
mod state;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tiles = [Tile::ShortGrass; TILES_COUNT];
    let individuals =
        std::array::from_fn(|i| Individual::new(Xy::from(XyIndex(i)), Behavior::MovingSouth));
    let world = World::new(tiles, individuals);

    let state = State::new(world);
    let network = Network::new();

    Runner::new(state, network).run()?;
    Ok(())
}
