use bevy::prelude::*;
use oc_geo::tile::{TileXy, WorldTileIndex};
use oc_physics::update::bevy::Region;
use oc_root::GEO_PIXELS_PER_TILE;

use crate::ingame::camera::move_::MovedBattleCamera;
use crate::ingame::draw;
use crate::world::tile::Tiles;

#[derive(Debug, Event)]
pub struct ToggleShowTiles;

#[derive(Debug, Deref, DerefMut, Resource, Default)]
pub struct ShowTiles(pub bool);

#[derive(Debug, Component)]
pub struct TileWire(WorldTileIndex);

pub fn on_toggle_show_tiles(
    _: On<ToggleShowTiles>,
    mut commands: Commands,
    mut showing: ResMut<ShowTiles>,
    wires: Query<(Entity, &TileWire)>,
    tiles: Res<Tiles>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    camera: Single<(&Camera, &GlobalTransform)>,
    window: Single<&Window>,
) -> Result {
    if showing.0 {
        for (entity, _) in wires {
            commands.entity(entity).despawn();
        }
    } else {
        let (camera, transform) = *camera;
        let window_width = window.width();
        let window_height = window.height();

        for (region, tiles) in tiles.iter() {
            for (i, _) in tiles {
                let tile: TileXy = (*i).into();
                let width = GEO_PIXELS_PER_TILE as f32;
                let height = GEO_PIXELS_PER_TILE as f32;
                let x = tile.0.0 as f32 * width + width / 2.;
                let y = tile.0.1 as f32 * height + height / 2.;
                let rectangle = Rectangle::new(width, height);
                let color = Color::srgba(0.1, 0.1, 0.6, 0.5);

                if let Ok(screen_position) =
                    camera.world_to_viewport(transform, Vec3::new(x, y, draw::Z_TILE_WIREFRAME))
                {
                    if !(screen_position.x >= 0.0
                        && screen_position.x <= window_width
                        && screen_position.y >= 0.0
                        && screen_position.y <= window_height)
                    {
                        continue;
                    }
                }

                commands.spawn((
                    TileWire(*i),
                    Region((*region).into()),
                    Mesh2d(meshes.add(rectangle)),
                    MeshMaterial2d(materials.add(color)),
                    Transform::from_xyz(x, y, draw::Z_TILE_WIREFRAME),
                ));
            }
        }
    }

    showing.0 = !showing.0;
    Ok(())
}

pub fn on_camera_moved(_: On<MovedBattleCamera>, mut commands: Commands, showing: Res<ShowTiles>) {
    if !showing.0 {
        return;
    }

    // Thats not an elegant way (not performant too ...) but we are in debug
    commands.trigger(ToggleShowTiles);
    commands.trigger(ToggleShowTiles);
}
