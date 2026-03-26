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
    states::{AppState, Config, InGameState, Meta, Mod},
};

#[cfg(feature = "debug")]
use debug::DebugPlugin;

#[cfg(feature = "debug")]
mod debug;
mod downloading;
mod entity;
mod error;
mod fx;
mod home;
mod ingame;
mod network;
mod setup;
mod states;
mod utils;
mod window;
mod world;

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct Args_ {
    #[clap(long)]
    pub autoconnect: Option<SocketAddr>,
}

#[derive(Resource)]
pub struct Args(pub Args_);

/// Spatial audio uses the distance to attenuate the sound volume. In 2D with the default camera,
/// 1 pixel is 1 unit of distance, so we use a scale so that 100 pixels is 1 unit of distance for
/// audio.
const AUDIO_SCALE: f32 = 1. / 100.0;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = App::new();

    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Open Combat".into(),
                    resolution: (800, 800).into(),
                    // present_mode: PresentMode::AutoNoVsync,
                    ..default()
                }),
                ..default()
            })
            .set(AudioPlugin {
                default_spatial_scale: SpatialScale::new_2d(AUDIO_SCALE),
                ..default()
            }),
    )
    .add_plugins(EguiPlugin::default())
    .add_plugins(Wireframe2dPlugin::default())
    .add_plugins(ErrorPlugin)
    .add_plugins(NetworkPlugin)
    .add_plugins(FxPlugin)
    .add_plugins(HomePlugin)
    .add_plugins(DownloadingPlugin)
    .add_plugins(IngamePlugin)
    .add_plugins(window::WindowPlugin)
    .add_plugins(ingame::camera::CameraPlugin)
    .insert_state(AppState::Home)
    .init_resource::<Mod>()
    .init_resource::<Meta>()
    .init_resource::<Config>()
    .init_resource::<states::Window>()
    .init_state::<InGameState>()
    .insert_resource(Args(Args_::parse()))
    .add_systems(Startup, setup::setup);

    #[cfg(feature = "debug")]
    app.add_plugins(DebugPlugin);

    app.run();

    Ok(())
}
