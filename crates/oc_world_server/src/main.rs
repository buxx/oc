use std::{net::SocketAddr, sync::Arc};

use clap::Parser;
use oc_geo::{
    region::WorldRegionIndex,
    tile::{TileXy, WorldTileIndex},
};
use oc_individual::{Individual, behavior::Behavior};
use oc_root::{GEO_PIXELS_PER_TILE, INDIVIDUALS_COUNT, TILES_COUNT};
use oc_world::{World, tile::Tile};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

use crate::{runner::Runner, state::State};

mod index;
mod individual;
mod network;
mod perf;
mod physics;
mod routing;
mod runner;
mod state;

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[clap(default_value = "0.0.0.0:6589")]
    pub host: SocketAddr,

    #[clap(long, action)]
    pub print_ticks: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    tracing_subscriber::fmt()
        .with_target(false)
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env()?,
        )
        .init();

    let tiles = vec![Tile::ShortGrass; TILES_COUNT];
    let individuals = individuals();
    let world = World::new(tiles, individuals);

    let (input, output) = network::listen(args.host);
    let state = Arc::new(State::new(world));

    // Blocking server logic
    Runner::new(state, output, args.print_ticks).run(input)?;

    Ok(())
}

fn individuals() -> Vec<Individual> {
    (0..INDIVIDUALS_COUNT)
        .map(|i| {
            let xy = TileXy::from(WorldTileIndex(i));
            let position = [
                ((xy.0.0 * GEO_PIXELS_PER_TILE) + GEO_PIXELS_PER_TILE / 2) as f32,
                ((xy.0.1 * GEO_PIXELS_PER_TILE) + GEO_PIXELS_PER_TILE / 2) as f32,
            ];
            let region: WorldRegionIndex = xy.into();
            Individual::new(position, xy, region, Behavior::Idle, vec![])
        })
        .collect()
}
