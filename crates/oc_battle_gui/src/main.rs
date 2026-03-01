use std::{net::SocketAddr, path::PathBuf};

use bevy::prelude::*;
use bevy::sprite_render::Wireframe2dPlugin;
use clap::Parser;

use crate::{
    downloading::DownloadingPlugin,
    error::ErrorPlugin,
    home::HomePlugin,
    ingame::IngamePlugin,
    network::NetworkPlugin,
    states::{AppState, InGameState},
};

#[cfg(feature = "debug")]
use debug::DebugPlugin;

#[cfg(feature = "debug")]
mod debug;
mod downloading;
mod entity;
mod error;
mod home;
mod ingame;
mod network;
mod setup;
mod states;

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct Args_ {
    #[clap(long)]
    pub autoconnect: Option<SocketAddr>,
}

#[derive(Resource)]
pub struct Args(pub Args_);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Open Combat".into(),
            resolution: (800, 800).into(),
            ..default()
        }),
        ..default()
    }))
    .add_plugins(Wireframe2dPlugin::default())
    .add_plugins(ErrorPlugin)
    .add_plugins(NetworkPlugin)
    .add_plugins(HomePlugin)
    .add_plugins(DownloadingPlugin)
    .add_plugins(IngamePlugin)
    .insert_state(AppState::Home)
    .init_state::<InGameState>()
    .insert_resource(Args(Args_::parse()))
    .add_systems(Startup, setup::setup);

    #[cfg(feature = "debug")]
    app.add_plugins(DebugPlugin);

    app.run();

    Ok(())
}
