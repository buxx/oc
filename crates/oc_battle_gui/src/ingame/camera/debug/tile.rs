use bevy::prelude::*;
use oc_geo::region::{RegionXy, WorldRegionIndex};
use oc_geo::tile::TileXy;
use oc_physics::update::bevy::Region;
use oc_root::GEO_PIXELS_PER_TILE;

use crate::ingame::region::{ForgottenRegion, ListeningRegion};
use crate::ingame::{camera, draw};
use crate::world::{InsertTiles, World};

#[derive(Debug, Event)]
pub struct ToggleShowTiles;

#[derive(Debug, Deref, DerefMut, Resource, Default)]
pub struct ShowTiles(pub bool);

#[derive(Debug, Event, Deref)]
pub struct SpawnRegionTiles(pub WorldRegionIndex);

#[derive(Debug, Event, Deref)]
pub struct DespawnRegionTiles(pub WorldRegionIndex);

// FIXME BS NOW; rename
#[derive(Debug, Component)]
pub struct TileWire;

pub fn on_toggle_show_tiles(
    _: On<ToggleShowTiles>,
    mut commands: Commands,
    state: ResMut<camera::State>,
    mut showing: ResMut<ShowTiles>,
    // wires: Query<(Entity, &TileWire)>,
    // world: Res<World>,
    // mut meshes: ResMut<Assets<Mesh>>,
    // mut materials: ResMut<Assets<ColorMaterial>>,
    // camera: Single<(&Camera, &GlobalTransform)>,
    // window: Single<&Window>,
) -> Result {
    showing.0 = !showing.0;

    if let Some(regions) = &state.regions {
        for region in regions {
            match showing.0 {
                true => commands.trigger(SpawnRegionTiles(region.0)),
                false => commands.trigger(DespawnRegionTiles(region.0)),
            }
        }
    }

    // if showing.0 {
    //     for (entity, _) in wires {
    //         commands.entity(entity).despawn();
    //     }
    // } else {
    //     let (camera, transform) = *camera;
    //     let window_width = window.width();
    //     let window_height = window.height();

    //     for (i, _) in world.tiles().iter() {
    //         let region: WorldRegionIndex = (**i).into();
    //         let tile: TileXy = (**i).into();
    //         let width = GEO_PIXELS_PER_TILE as f32;
    //         let height = GEO_PIXELS_PER_TILE as f32;
    //         let x = tile.0.0 as f32 * width + width / 2.;
    //         let y = tile.0.1 as f32 * height + height / 2.;
    //         let rectangle = Rectangle::new(width, height);
    //         let color = Color::srgba(0.1, 0.1, 0.6, 0.5);

    //         if let Ok(screen_position) =
    //             camera.world_to_viewport(transform, Vec3::new(x, y, draw::Z_TILE_WIREFRAME))
    //         {
    //             if !(screen_position.x >= 0.0
    //                 && screen_position.x <= window_width
    //                 && screen_position.y >= 0.0
    //                 && screen_position.y <= window_height)
    //             {
    //                 continue;
    //             }
    //         }

    //         commands.spawn((
    //             TileWire,
    //             Region(region.into()),
    //             // FIXME BS NOW: sprite from terrain.png
    //             Mesh2d(meshes.add(rectangle)),
    //             MeshMaterial2d(materials.add(color)),
    //             Transform::from_xyz(x, y, draw::Z_TILE_WIREFRAME),
    //         ));
    //     }
    // }

    Ok(())
}

pub fn on_insert_tiles(tiles: On<InsertTiles>, mut commands: Commands) {
    commands.trigger(SpawnRegionTiles(tiles.0));
}

pub fn on_spawn_region_tiles(
    region: On<SpawnRegionTiles>,
    mut commands: Commands,
    showing: Res<ShowTiles>,
) {
    if !showing.0 {
        return;
    }

    tracing::info!("SPAWN {:?}", region.0);
}

pub fn on_forgotten_region(region: On<ForgottenRegion>, mut commands: Commands) {
    commands.trigger(DespawnRegionTiles(region.0));
}

pub fn on_despawn_region_tiles(
    region: On<DespawnRegionTiles>,
    mut commands: Commands,
    tiles: Query<(&TileWire, Entity, &Region)>,
) {
    let region: RegionXy = region.0.into();
    tracing::info!("DESPAWN {:?}", region.0);

    for (_, entity, region_) in tiles {
        if region_.0 == region {
            commands.entity(entity).despawn();
        }
    }
}
