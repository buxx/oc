use std::net::SocketAddr;

use bevy::prelude::*;
use clap::Parser;

use crate::config::{Config_, Connect};

mod config;
#[cfg(feature = "debug")]
mod debug;
mod downloading;
mod entity;
mod error;
mod fx;
mod home;
mod individual;
mod ingame;
mod network;
#[cfg(feature = "debug")]
mod projectile;
mod run;
mod setup;
mod sprites;
mod states;
#[cfg(feature = "debug")]
mod tileset;
mod utils;
mod window;
mod world;

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[clap(long)]
    pub autoconnect: Option<SocketAddr>,
}

impl From<Args> for Config_ {
    fn from(value: Args) -> Self {
        let autoconnect = value.autoconnect.map(Connect::Network);
        Self { autoconnect }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    run::run().config(Args::parse().into()).call();
    Ok(())
}
