use std::path::PathBuf;

use bevy::prelude::*;
use oc_geo::region::{RegionXy, WorldRegionIndex};
use oc_root::{REGION_HEIGHT_PIXELS, REGION_WIDTH_PIXELS};

use crate::{
    entity::world::region::{Region, RegionBackground},
    ingame::draw::Z_REGION_BACKGROUND,
    states::Meta,
};

#[cfg(feature = "debug")]
pub mod debug;

#[derive(Debug, Event)]
pub struct ListeningRegion(pub WorldRegionIndex);

#[derive(Debug, Event)]
pub struct ForgottenRegion(pub WorldRegionIndex);

pub fn on_listening_region(
    region: On<ListeningRegion>,
    mut commands: Commands,
    assets: Res<AssetServer>,
    meta: Res<Meta>,
) {
    let Some(meta) = &meta.0 else { return };
    tracing::debug!("Spawn region background {:?}", region.0);

    let region = region.0;
    // TODO: normalize somewhere
    let path = PathBuf::from(".cache").join("maps");
    let path = path.join(meta.folder_name());
    let path = path.join(region.background_file_name());
    let region: RegionXy = region.into();

    let width = REGION_WIDTH_PIXELS as f32;
    let height = REGION_HEIGHT_PIXELS as f32;
    let x = region.0.0 as f32 * width;
    let y = region.0.1 as f32 * height;
    let x = x + width / 2.;
    let y = y + height / 2.;

    tracing::trace!(name="spawn-region-background", region=?region, x=x, y=y, path=?path);
    commands.spawn((
        RegionBackground,
        Region(region),
        Sprite::from_image(assets.load(path)),
        Transform {
            scale: Vec3::new(1.0, -1.0, 1.0), // Mirror on Y-axis
            translation: Vec3::new(x as f32, y as f32, Z_REGION_BACKGROUND),
            ..default()
        },
    ));
}

pub fn on_forgotten_region(
    region: On<ListeningRegion>,
    mut commands: Commands,
    query: Query<(Entity, &Region), With<RegionBackground>>,
) {
    let region: RegionXy = region.0.into();
    for (entity, region_) in query {
        if region_.0 == region {
            commands.entity(entity).despawn();
        }
    }
}
