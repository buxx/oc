use bevy::prelude::*;

pub fn setup(mut commands: Commands, _asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);
}
