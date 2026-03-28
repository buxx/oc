use std::net::SocketAddr;

use bevy::audio::{AudioPlugin, SpatialScale};
use bevy::prelude::*;
use bevy::sprite_render::Wireframe2dPlugin;
use bevy_egui::EguiPlugin;
use clap::Parser;

use crate::{
    downloading::DownloadingPlugin,
    error::ErrorPlugin,
    fx::FxPlugin,
    home::HomePlugin,
    ingame::IngamePlugin,
    network::NetworkPlugin,
    states::{AppState, InGameState, Meta, Mod, StaticSource},
};

mod config;
#[cfg(feature = "debug")]
mod debug;
mod downloading;
mod entity;
mod error;
mod fx;
mod home;
mod ingame;
mod network;
mod run;
mod setup;
mod states;
mod utils;
mod window;
mod world;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    run::run(config::Config_::default())
}
