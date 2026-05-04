use std::f32::consts::PI;
use std::path::PathBuf;

use bevy::{
    color::palettes::css::WHITE,
    input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll},
    prelude::*,
};
use bevy_heightmap::{HeightMap, HeightMapPlugin, ValueFunctionHeightMap};
use clap::Parser;
use oc_root::WorldConfig;
use oc_world::reader::MapReader;

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
struct Args {
    #[clap()]
    world: PathBuf,
}

fn main() {
    let args = Args::parse();

    let init = move |mut commands: Commands| {
        commands.trigger(Spawn(args.world.clone()));
    };

    let mut app = App::new();
    app.add_plugins((DefaultPlugins, HeightMapPlugin))
        .init_resource::<CameraOrbit>()
        .add_observer(on_spawn)
        .add_systems(Startup, (setup, init))
        .add_systems(Update, camera_control)
        .run();
}
