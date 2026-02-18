use std::net::SocketAddr;

use bevy::prelude::*;
use clap::Parser;

use crate::{
    error::ErrorPlugin, home::HomePlugin, ingame::IngamePlugin, loading::LoadingPlugin,
    network::NetworkPlugin, states::AppState,
};

mod error;
mod home;
mod ingame;
mod loading;
mod network;
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
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ErrorPlugin)
        .add_plugins(NetworkPlugin)
        .add_plugins(HomePlugin)
        .add_plugins(LoadingPlugin)
        .add_plugins(IngamePlugin)
        .insert_state(AppState::Home)
        .insert_resource(Args(Args_::parse()))
        .run();

    Ok(())
}
