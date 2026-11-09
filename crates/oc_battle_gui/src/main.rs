// derive_more::Constructor triggers a false positive
#![allow(clippy::redundant_field_names)]

use std::{net::SocketAddr, path::PathBuf};

use bevy::prelude::*;
use clap::Parser;
use oc_root::{files::Roots, side::Side};

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
    /// Directory where files downloaded from a server are stored (default: user data directory)
    #[clap(long)]
    pub data_dir: Option<PathBuf>,
}

impl TryFrom<Args> for Config_ {
    type Error = std::io::Error;

    fn try_from(value: Args) -> Result<Self, Self::Error> {
        let auto_connect = value
            .auto_connect
            .map(|addr| (Connect::Network(addr), value.auto_connect_side));
        let data_dir = value.data_dir.unwrap_or_else(Roots::default_data_dir);
        let roots = Roots::under(&data_dir)?;
        Ok(Self::builder()
            .maybe_auto_connect(auto_connect)
            .roots(roots)
            .build())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    run::run()
        .config(Args::parse().try_into()?)
        .logging(std::env::var("RUST_LOG").is_ok())
        .call();
    Ok(())
}
