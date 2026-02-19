use std::net::SocketAddr;

use bevy::sprite_render::Wireframe2dPlugin;
use bevy::{
    dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin, FrameTimeGraphConfig},
    prelude::*,
    text::FontSmoothing,
};
use clap::Parser;

use crate::{
    error::ErrorPlugin, home::HomePlugin, ingame::IngamePlugin, loading::LoadingPlugin,
    network::NetworkPlugin, states::AppState,
};

mod entity;
mod error;
mod home;
mod ingame;
mod loading;
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
    App::new()
        .add_plugins((
            DefaultPlugins,
            FpsOverlayPlugin {
                config: FpsOverlayConfig {
                    text_config: TextFont {
                        // Here we define size of our overlay
                        font_size: 42.0,
                        // If we want, we can use a custom font
                        font: default(),
                        // We could also disable font smoothing,
                        font_smoothing: FontSmoothing::default(),
                        ..default()
                    },
                    // We can also change color of the overlay
                    text_color: Color::srgb(128.0, 128.0, 128.0),
                    // We can also set the refresh interval for the FPS counter
                    refresh_interval: core::time::Duration::from_millis(100),
                    enabled: true,
                    frame_time_graph_config: FrameTimeGraphConfig {
                        enabled: true,
                        // The minimum acceptable fps
                        min_fps: 30.0,
                        // The target fps
                        target_fps: 144.0,
                    },
                },
            },
        ))
        .add_plugins(Wireframe2dPlugin::default())
        .add_plugins(ErrorPlugin)
        .add_plugins(NetworkPlugin)
        .add_plugins(HomePlugin)
        .add_plugins(LoadingPlugin)
        .add_plugins(IngamePlugin)
        .insert_state(AppState::Home)
        .insert_resource(Args(Args_::parse()))
        .add_systems(Startup, setup::setup)
        .run();

    Ok(())
}
