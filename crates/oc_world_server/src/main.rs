use std::{net::SocketAddr, path::PathBuf, sync::Arc};

use clap::Parser;
use oc_geo::{
    region::RegionXy,
    tile::{TileXy, WorldTileIndex},
};
use oc_individual::{Individual, behavior::Behavior};
use oc_root::{GEO_PIXELS_PER_TILE, INDIVIDUALS_COUNT, TILES_COUNT};
use oc_world::{World, load::WorldLoader, tile::Tile};
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

    #[clap()]
    pub world: PathBuf,

    #[clap(long, action)]
    pub print_ticks: bool,

    #[clap(long, default_value = ".cache")]
    pub cache: PathBuf,
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

    let world = WorldLoader::new(args.world.clone(), args.cache.clone()).load()?;
    let (input, output) = network::listen(args.host);
    let state = Arc::new(State::new(world));

    // Blocking server logic
    Runner::new(state, output, args.print_ticks).run(input)?;

    Ok(())
}
