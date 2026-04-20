use bevy::audio::{AudioPlugin, SpatialScale};
use bevy::prelude::*;
use bevy::sprite_render::Wireframe2dPlugin;
use bevy_egui::EguiPlugin;

use crate::config::{Config, Config_};
#[cfg(feature = "debug")]
use crate::debug;
use crate::{
    downloading::DownloadingPlugin,
    error::ErrorPlugin,
    fx::FxPlugin,
    home::HomePlugin,
    ingame::IngamePlugin,
    network::NetworkPlugin,
    states::{AppState, InGameState, Meta, Mod, StaticSource},
};
use crate::{ingame, setup, states, window};

/// Spatial audio uses the distance to attenuate the sound volume. In 2D with the default camera,
/// 1 pixel is 1 unit of distance, so we use a scale so that 100 pixels is 1 unit of distance for
/// audio.
const AUDIO_SCALE: f32 = 1. / 100.0;

#[cfg(feature = "debug")]
use debug::DebugPlugin;

pub fn run(config: Config_) {
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
    .init_resource::<StaticSource>()
    .init_resource::<states::Window>()
    .init_state::<InGameState>()
    .insert_resource(Config(config))
    .add_systems(Startup, setup::setup);

    #[cfg(feature = "debug")]
    app.add_plugins(DebugPlugin);

    tracing::info!("Start app");
    app.run();
}
