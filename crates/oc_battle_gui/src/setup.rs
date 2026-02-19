use bevy::{color::palettes::css::PURPLE, prelude::*};

pub fn setup(
    mut commands: Commands,
    _asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);
    // commands.spawn(Sprite::from_image(
    //     asset_server.load("branding/bevy_bird_dark.png"),
    // ));
    commands.spawn((
        Mesh2d(meshes.add(Circle::new(5.0))),
        MeshMaterial2d(materials.add(Color::from(PURPLE))),
        Transform::from_xyz(50.0, 50.0, 1.0),
    ));
}
