use std::{net::SocketAddr, path::PathBuf, sync::Arc};

use clap::Parser;
use oc_mod::Mod;
use oc_root::{config::Config, ids::Ids};
use oc_world::load::WorldLoader;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

use crate::{runner::Runner, state::State, static_::Static};

mod index;
mod individual;
mod network;
mod perf;
mod physics;
mod routing;
mod runner;
mod state;
mod static_;
mod utils;

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[clap(long, default_value = "0.0.0.0:6589")]
    pub host: SocketAddr,

    #[clap(long("static"), default_value = "0.0.0.0:6590")]
    pub static_: SocketAddr,

    #[clap()]
    pub world: PathBuf,

    #[clap()]
    pub mod_: PathBuf,

    #[clap(long, action)]
    pub print_ticks: bool,

    #[clap(long, default_value = ".cache")]
    pub cache: PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let cache = args.cache.clone();
    let world = args.world.clone();
    let mod_ = args.mod_.clone();

    tracing_subscriber::fmt()
        .with_target(false)
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env()?,
        )
        .init();

    let ids = Ids::default();
    let mod_ = Mod::load(&mod_, Some(&cache))?;
    let world = WorldLoader::new(mod_.clone(), world.clone(), cache.clone()).load(&ids)?;
    let (input, output) = network::listen(args.host);
    let state = Arc::new(State::new(ids, world));
    let config = Config::new(args.static_.port());

    let state_ = state.clone();
    std::thread::spawn(move || Static::new(state_, args.cache).serve(args.static_));

    Runner::new(config, mod_, state, output, args.print_ticks).run(input)?;

    Ok(())
}
