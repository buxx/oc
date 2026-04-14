use bevy::prelude::*;
use bevy::sprite::Anchor;
use oc_geo::region::{RegionXy, WorldRegionIndex};
use oc_geo::tile::{TileXy, WorldTileIndex};
use oc_physics::update::bevy::Region;
use oc_root::y::Y;
use oc_root::{GEO_PIXELS_PER_TILE, files};
use oc_utils::bevy::EntityMapping;
use oc_utils::d2::Xy;

use crate::ingame::camera::{self, State};
use crate::ingame::draw::Z_TERRAIN_TILE;
use crate::ingame::region::ForgottenRegion;
use crate::network;
use crate::states::{Meta, Mod, StaticSource};
use crate::world::{InsertedTiles, World};

#[derive(Debug, Event)]
pub struct ToggleShowTiles;

#[derive(Debug, Deref, DerefMut, Resource, Default)]
pub struct ShowTiles(pub bool);

#[derive(Debug, Event, Deref)]
pub struct SpawnRegionTiles(pub WorldRegionIndex);

#[derive(Debug, Event, Deref)]
pub struct DespawnRegionTiles(pub WorldRegionIndex);

#[derive(Debug, Component)]
pub struct TerrainTile(WorldTileIndex);

pub fn on_toggle_show_tiles(
    _: On<ToggleShowTiles>,
    mut commands: Commands,
    state: ResMut<camera::State>,
    mut showing: ResMut<ShowTiles>,
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

    Ok(())
}

pub fn on_insert_tiles(tiles: On<InsertedTiles>, mut commands: Commands) {
    commands.trigger(SpawnRegionTiles(tiles.0));
}

pub fn on_spawn_region_tiles(
    region: On<SpawnRegionTiles>,
    mut commands: Commands,
    showing: Res<ShowTiles>,
    mod_: Res<Mod>,
    meta: Res<Meta>,
    static_: Res<StaticSource>,
    network: Res<network::state::State>,
    world_: Res<World>,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut entities: ResMut<EntityMapping<WorldTileIndex>>,
) {
    if !showing.0 {
        return;
    }

    let (Some(mod_), Some(meta), Some(static_), Some(connect), Some(terrain)) = (
        &mod_.0,
        &meta.0,
        &static_.0,
        &network.server,
        &world_.terrain,
    ) else {
        return;
    };
    tracing::info!("Spawn region {:?} tiles", region.0);

    let mod_ = mod_.canonical();
    let world = meta.canonical();
    let files = files::Files::new(mod_, world).into_gui(static_.clone(), connect.clone().into());
    let terrain_png = files.terrain_png().display().to_string();

    let texture = asset_server.load(&terrain_png);
    let layout = terrain.layout();
    let texture_atlas_layout = texture_atlas_layouts.add(layout);

    if let Some(tiles) = world_.tiles().get(&region) {
        for (i, tile) in tiles {
            let xy: Xy = (*i).into();
            // TODO (map terrain should be checked to avoid manage missing terrain here)
            let index = (*terrain.natures.get(&tile.nature).unwrap()) as usize;
            let x = xy.0 * GEO_PIXELS_PER_TILE;
            let y = xy.1 * GEO_PIXELS_PER_TILE;
            let point = Vec3::new(x as f32, (y as f32).to_gui_y(), Z_TERRAIN_TILE);

            // FIXME BS NOW
            // if region.0.0 == 0 {
            //     if xy.1 == 99 {
            //         tracing::error!("spawn {} at {:?}", xy.1, (x, y));
            //     }
            // }

            let entity = commands
                .spawn((
                    TerrainTile(*i),
                    Region(region.0.into()),
                    Sprite {
                        image: texture.clone(),
                        texture_atlas: Some(TextureAtlas {
                            layout: texture_atlas_layout.clone(),
                            index,
                        }),
                        ..Default::default()
                    },
                    Transform::from_translation(point),
                    Anchor::TOP_LEFT,
                ))
                .id();
            entities.insert(*i, entity);
        }
    }
}

pub fn on_forgotten_region(region: On<ForgottenRegion>, mut commands: Commands) {
    commands.trigger(DespawnRegionTiles(region.0));
}

pub fn on_despawn_region_tiles(
    region: On<DespawnRegionTiles>,
    mut commands: Commands,
    tiles: Query<(&TerrainTile, Entity, &Region)>,
    mut entities: ResMut<EntityMapping<WorldTileIndex>>,
) {
    tracing::info!("Despawn region {:?} tiles", region.0);
    let region: RegionXy = region.0.into();

    for (tile, entity, region_) in tiles {
        if region_.0 == region {
            entities.remove(&tile.0);
            commands.entity(entity).despawn();
        }
    }
}

pub fn tile_under_cursor(
    mut state: ResMut<State>,
    window_: Single<&Window>,
    camera_: Single<(&Camera, &GlobalTransform)>,
    entities: Res<EntityMapping<WorldTileIndex>>,
    mut tiles: Query<(&TerrainTile, &mut Sprite)>,
) {
    let (camera, transform) = *camera_;
    if let Some(cursor) = window_.cursor_position() {
        if let Ok(cursor) = camera.viewport_to_world_2d(transform, cursor) {
            let point = Vec2::new(cursor.x, cursor.y.to_gui_y());
            let tile: TileXy = [point.x, point.y].into();
            let current: WorldTileIndex = tile.into();

            match state.tile {
                Some(previous) => {
                    if previous != current {
                        if let Some(previous) = entities.get(&previous) {
                            if let Ok((_, mut sprite)) = tiles.get_mut(*previous) {
                                sprite.color = Color::WHITE;
                            }
                        }

                        if let Some(current) = entities.get(&current) {
                            if let Ok((_, mut sprite)) = tiles.get_mut(*current) {
                                sprite.color = Color::BLACK;
                            }
                        }

                        tracing::info!("{:?}", tile);
                        state.tile = Some(current);
                    }
                }
                None => state.tile = Some(current),
            }
        };
    };
}
