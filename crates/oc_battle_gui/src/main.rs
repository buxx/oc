use std::net::SocketAddr;

use bevy::prelude::*;
use clap::Parser;
use oc_root::side::Side;

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
mod menu;
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

#[cfg(test)]
pub mod tests;

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[clap(long)]
    pub auto_connect: Option<SocketAddr>,
    #[clap(long, value_enum, default_value = "a")]
    pub auto_connect_side: Side,
}

impl From<Args> for Config_ {
    fn from(value: Args) -> Self {
        let auto_connect = value
            .auto_connect
            .map(|addr| (Connect::Network(addr), value.auto_connect_side));
        Self::builder().maybe_auto_connect(auto_connect).build()
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    run::run().config(Args::parse().into()).call();
    Ok(())
}
