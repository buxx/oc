use std::{
    net::SocketAddr,
    path::PathBuf,
    sync::{Arc, mpsc::channel},
};

use clap::Parser;
use message_io::network::Endpoint;
use oc_root::{files, static_::StaticSource};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

#[cfg(feature = "tracker")]
use crate::tracker::Tracker;
use crate::{config::ServerConfig, network::NetworkConfig, runner::Runner, static_::Static};

mod bridge;
mod config;
mod index;
mod individual;
mod network;
#[cfg(feature = "perfs")]
mod perf;
mod physics;
mod projectile;
mod routing;
mod runner;
mod schedule;
mod state;
mod static_;
#[cfg(feature = "tracker")]
pub mod tracker;
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

    let files = files::Files::new("".to_string(), "".to_string()).into_server(args.cache.clone());
    std::fs::create_dir_all(files.mods()).unwrap(); // TODO
    std::fs::create_dir_all(files.worlds()).unwrap(); // TODO

    #[cfg(feature = "tracker")]
    let tracker = Tracker::default();

    let network: NetworkConfig = args.clone().into();
    let config: ServerConfig = args.clone().into();
    let state = state::init::<Endpoint>(config.clone())?;
    let state = Arc::new(state);

    let (input, output) = network::listen(network.clone());
    {
        let state = state.clone();
        let config = config.clone();
        std::thread::spawn(move || Static::new(state, network.clone(), config).serve(args.static_));
    }

    let (ready, _) = channel();
    let (stop_tx, stop_rx) = channel();
    std::thread::spawn(move || {
        Runner::new(
            config,
            state,
            output,
            #[cfg(feature = "tracker")]
            tracker,
        )
        .run(input, ready, stop_rx);
    });

    let (tx, rx) = channel();
    ctrlc::set_handler(move || {
        tx.send(()).expect("Could not send signal on channel.");
    })
    .expect("Error setting Ctrl-C handler");
    tracing::info!("Waiting for SIGINT (Ctrl+C)...");
    let _ = rx.recv();

    tracing::info!("SIGINT (Ctrl+C) received, stop runner");
    let _ = stop_tx.send(());

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
            static_: StaticSource::Remote(value.static_.port()),
            snapshot: value.snapshot,
        }
    }
}

impl From<Args> for NetworkConfig {
    fn from(value: Args) -> Self {
        Self {
            host: value.host,
            // static_: value.static_.clone(),
            // cache: value.cache.clone(),
        }
    }
}
