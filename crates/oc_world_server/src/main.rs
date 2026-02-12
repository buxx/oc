use std::{net::SocketAddr, sync::Arc};

use clap::Parser;
use oc_individual::{Individual, behavior::Behavior};
use oc_network::ToClient;
use oc_root::{INDIVIDUALS_COUNT, TILES_COUNT};
use oc_utils::d2::{Xy, XyIndex};
use oc_world::{World, tile::Tile};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

use crate::{network::Network, runner::Runner, state::State};

mod index;
mod individual;
mod network;
mod perf;
mod runner;
mod state;

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[clap(default_value = "0.0.0.0:6589")]
    pub host: SocketAddr,
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
    let individuals = (0..INDIVIDUALS_COUNT)
        .map(|i| Individual::new(Xy::from(XyIndex(i)), Behavior::MovingSouth))
        .collect();
    let world = World::new(tiles, individuals);

    let state = Arc::new(State::new(world));
    let (network, input) = Network::listen(args.host);
    let network = Arc::new(network);

    {
        let network = network.clone();
        std::thread::spawn(move || {
            while let Ok(message) = input.recv() {
                tracing::info!("DEBUG Message received");
                match message {
                    network::Event::Connected(endpoint) => {}
                    network::Event::Disconnected(endpoint) => {}
                    network::Event::Message(endpoint, to_server) => {
                        network.send(vec![endpoint], ToClient::Hello);
                    }
                }
            }
        });
    }

    // Blocking server logic
    Runner::new(state, network).run()?;

    Ok(())
}
