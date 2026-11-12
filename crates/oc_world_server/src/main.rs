// derive_more::Constructor triggers a false positive
#![allow(clippy::redundant_field_names)]

use std::{
    net::SocketAddr,
    path::PathBuf,
    sync::{Arc, mpsc::channel},
};

use clap::Parser;
use message_io::network::Endpoint;
use oc_root::{files::Roots, static_::StaticSource};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

#[cfg(feature = "tracker")]
use crate::tracker::Tracker;
use crate::{
    config::ServerConfig, end::EndPolicy, network::NetworkConfig, runner::Runner, static_::Static,
};

mod bridge;
mod config;
mod end;
mod index;
mod individual;
mod network;
mod outline;
#[cfg(feature = "perfs")]
mod perf;
mod physics;
mod projectile;
mod routing;
mod runner;
mod schedule;
mod squad;
mod state;
mod static_;
#[cfg(feature = "tracker")]
pub mod tracker;
mod utils;
pub mod visibility;
mod world;

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

    #[clap(long)]
    pub region_width: u64,

    #[clap(long)]
    pub region_height: u64,

    #[clap(long, action)]
    pub print_ticks: bool,

    #[clap(long, default_value = ".cache")]
    pub cache: PathBuf,

    /// What to do when battle ends: `none` or `save-map-persist-to:<path>`
    #[clap(long, default_value = "none")]
    pub end: EndPolicy,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    setup_logging()?;

    #[cfg(feature = "tracker")]
    let tracker = Tracker::default();

    let network: NetworkConfig = args.clone().into();
    let config = args.server_config()?;
    config.roots.ensure_server_dirs()?;

    let map = oc_world::reader::MapReader::new(&args.world)?;
    let state = state::init::<Endpoint>(config.clone(), &map)?;
    let state = Arc::new(state);

    let (input, output) = network::listen(network.clone());
    {
        let state = state.clone();
        let config = config.clone();
        std::thread::spawn(move || Static::new(state, network.clone(), config).serve(args.static_));
    }

    let (ready, _) = channel();
    let (stop_tx, stop_rx) = channel();
    let end = args.end.clone();
    std::thread::spawn(move || {
        Runner::new(
            config,
            state,
            output,
            #[cfg(feature = "tracker")]
            tracker,
            #[cfg(feature = "debug")]
            None,
            end,
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

impl Args {
    fn server_config(&self) -> std::io::Result<ServerConfig> {
        let parent = |path: &PathBuf| -> std::io::Result<PathBuf> {
            Ok(std::path::absolute(path)?
                .parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_default())
        };
        let roots = Roots::in_place(&parent(&self.mod_)?, &parent(&self.world)?, &self.cache)?;

        Ok(ServerConfig {
            world: self.world.clone(),
            mod_: self.mod_.clone(),
            roots,
            static_: StaticSource::Remote(self.static_.port()),
            snapshot: self.snapshot.clone(),
            region_width: self.region_width,
            region_height: self.region_height,
        })
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
