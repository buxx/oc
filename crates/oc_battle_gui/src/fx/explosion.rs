use bevy::prelude::*;
use oc_network::GameConfig;
use oc_root::{
    WcfgFrom,
    files::FilesAsGui,
    geo::{ScreenVec2, WorldVec2},
    physics::Meters,
};
use oc_utils::let_some;

use crate::ingame::draw::Z_EXPLOSION;

/// Frames of "explosions.png" sprite sheet (one row)
const FRAME_SIZE: UVec2 = UVec2::new(196, 190);
const FRAMES: u32 = 11;
const FRAME_DURATION: f32 = 0.15;

#[derive(Component)]
pub struct ExplosionAnimation(Timer);

pub fn spawn(
    commands: &mut Commands,
    assets: &AssetServer,
    layouts: &mut Assets<TextureAtlasLayout>,
    g: &GameConfig,
    files: &FilesAsGui,
    position: WorldVec2,
    rayon: Meters,
) {
    let layout = layouts.add(TextureAtlasLayout::from_grid(
        FRAME_SIZE, FRAMES, 1, None, None,
    ));
    let sprite_path = files.sprites().join("explosions.png");
    tracing::debug!(name = "fx-explosion-spawn", path=?sprite_path, position=?position);
    let image = assets.load(sprite_path);
    let position = ScreenVec2::from_(position, &g.w);
    let width = rayon.pixels(&g.w) * 2.;
    let height = width * FRAME_SIZE.y as f32 / FRAME_SIZE.x as f32;

    commands.spawn((
        Sprite {
            custom_size: Some(Vec2::new(width, height)),
            ..Sprite::from_atlas_image(image, TextureAtlas { layout, index: 0 })
        },
        Transform::from_xyz(position.x, position.y, Z_EXPLOSION),
        ExplosionAnimation(Timer::from_seconds(FRAME_DURATION, TimerMode::Repeating)),
    ));
}

pub fn animate(
    mut commands: Commands,
    time: Res<Time>,
    mut explosions: Query<(Entity, &mut ExplosionAnimation, &mut Sprite)>,
) {
    for (entity, mut animation, mut sprite) in &mut explosions {
        if !animation.0.tick(time.delta()).just_finished() {
            continue;
        }
        let_some!(atlas = &mut sprite.texture_atlas, continue);
        if atlas.index + 1 >= FRAMES as usize {
            commands.entity(entity).despawn();
        } else {
            atlas.index += 1;
        }
    }
}
