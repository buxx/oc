use std::{net::SocketAddr, path::PathBuf, sync::Arc};

use clap::Parser;
use message_io::network::Endpoint;
use oc_root::static_::StaticSource;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

use crate::{config::ServerConfig, network::NetworkConfig, static_::Static};

mod bridge;
mod config;
mod index;
mod individual;
mod network;
mod perf;
mod physics;
mod projectile;
mod routing;
mod run;
mod runner;
mod schedule;
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
    pub mod_: PathBuf,

    #[clap()]
    pub world: PathBuf,

    #[clap()]
    pub snapshot: PathBuf,

    #[clap(long, action)]
    pub print_ticks: bool,

    #[clap(long, default_value = ".cache")]
    pub cache: PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    setup_logging()?;

    let network: NetworkConfig = args.clone().into();
    let config: ServerConfig = args.clone().into();
    let state = state::init::<Endpoint>(config.clone())?;
    let state = Arc::new(state);

    let (input, output) = network::listen(network.clone());
    {
        let state = state.clone();
        std::thread::spawn(move || Static::new(state, network.clone()).serve(args.static_));
    }

    let (ready, _) = channel();
    run::run(config, state, input, output, ready);
    Ok(())
}

fn setup_logging() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env()?,
        )
        .init();
    Ok(())
}

impl From<Args> for ServerConfig {
    fn from(value: Args) -> Self {
        Self {
            world: value.world.clone(),
            mod_: value.mod_.clone(),
            cache: value.cache.clone(),
            print_ticks: value.print_ticks,
            static_: StaticSource::Remote(value.static_.port()),
            snapshot: value.snapshot,
        }
    }
}

impl From<Args> for NetworkConfig {
    fn from(value: Args) -> Self {
        Self {
            host: value.host.clone(),
            static_: value.static_.clone(),
            cache: value.cache.clone(),
        }
    }
}
