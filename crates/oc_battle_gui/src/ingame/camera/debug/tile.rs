use std::hash::Hash;

use bevy::prelude::*;
use bevy::sprite::Anchor;
use oc_geo::region::WorldRegionIndex;
use oc_geo::tile::{TileXy, WorldHeightIndex, WorldTileIndex};
use oc_root::y::Y;
use oc_root::{Wcfg, WcfgFrom, WcfgInto, files};
use oc_utils::bevy::EntityMapping;
use oc_utils::d2::Xy;
use oc_world::terrain::Terrain;
use oc_world::tile::Tile;

use crate::ingame::camera::{self, State};
use crate::ingame::region::ForgottenRegion;
use crate::network;
use crate::states::{Meta, Mod, StaticSource};
use crate::tileset::ConcernedTileset;
use crate::world::{InsertedTiles, World};

#[derive(Debug, Event)]
pub struct ToggleShowTiles;

#[derive(Debug, Default)]
pub enum ShowTileMode {
    #[default]
    None,
    Terrain,
    Height,
}

#[derive(Debug, Deref, DerefMut, Resource, Default)]
pub struct ShowTiles(pub ShowTileMode);

#[derive(Debug, Event, Deref)]
pub struct SpawnRegionTiles(pub WorldRegionIndex);

#[derive(Debug, Event, Deref)]
pub struct DespawnRegionTiles(pub WorldRegionIndex);

#[derive(Debug, Event, Deref)]
pub struct SpawnRegionHeights(pub WorldRegionIndex);

#[derive(Debug, Event, Deref)]
pub struct DespawnRegionHeights(pub WorldRegionIndex);

#[derive(Debug, Component)]
pub struct TerrainTile(WorldTileIndex);

impl oc_geo::region::Region for SpawnRegionTiles {
    fn region(&self) -> WorldRegionIndex {
        self.0
    }

    fn set_region(&mut self, value: WorldRegionIndex) {
        self.0 = value
    }
}

impl oc_geo::region::Region for DespawnRegionTiles {
    fn region(&self) -> WorldRegionIndex {
        self.0
    }

    fn set_region(&mut self, value: WorldRegionIndex) {
        self.0 = value
    }
}

impl ConcernedTileset<WorldTileIndex, Tile, Terrain> for SpawnRegionTiles {
    fn tileset<'a>(&self, world: &'a World) -> &'a Option<Terrain> {
        &world.terrain
    }
}

impl oc_geo::region::Region for SpawnRegionHeights {
    fn region(&self) -> WorldRegionIndex {
        self.0
    }

    fn set_region(&mut self, value: WorldRegionIndex) {
        self.0 = value
    }
}

impl ConcernedTileset<WorldHeightIndex, u8, Terrain> for SpawnRegionHeights {
    fn tileset<'a>(&self, world: &'a World) -> &'a Option<Terrain> {
        &world.terrain
    }
}

impl oc_geo::region::Region for DespawnRegionHeights {
    fn region(&self) -> WorldRegionIndex {
        self.0
    }

    fn set_region(&mut self, value: WorldRegionIndex) {
        self.0 = value
    }
}

pub fn on_toggle_show_tiles(
    _: On<ToggleShowTiles>,
    mut commands: Commands,
    state: ResMut<camera::State>,
    mut showing: ResMut<ShowTiles>,
) -> Result {
    showing.0 = match showing.0 {
        ShowTileMode::None => ShowTileMode::Terrain,
        ShowTileMode::Terrain => ShowTileMode::Height,
        ShowTileMode::Height => ShowTileMode::None,
    };

    if let Some(regions) = &state.regions {
        for region in regions {
            match showing.0 {
                ShowTileMode::None => {
                    commands.trigger(DespawnRegionHeights(region.0));
                }
                ShowTileMode::Terrain => {
                    commands.trigger(SpawnRegionTiles(region.0));
                }
                ShowTileMode::Height => {
                    commands.trigger(DespawnRegionTiles(region.0));
                    commands.trigger(SpawnRegionHeights(region.0));
                }
            }
        }
    }

    Ok(())
}

pub fn on_insert_tiles(tiles: On<InsertedTiles>, mut commands: Commands, showing: Res<ShowTiles>) {
    match showing.0 {
        ShowTileMode::None => {}
        ShowTileMode::Terrain => commands.trigger(SpawnRegionTiles(tiles.0)),
        ShowTileMode::Height => commands.trigger(SpawnRegionHeights(tiles.0)),
    };
}

// TODO: L'idée, a terme, est que ce generique soit utilisé pour afficher les decors aussi
// FIXME: use associated types to simplify
pub fn on_spawn_region<'a, E, I, T, S>(
    event: On<E>,
    mut commands: Commands,
    w: Res<Wcfg>,
    mod_: Res<Mod>,
    meta: Res<Meta>,
    static_: Res<StaticSource>,
    network: Res<network::state::State>,
    world_: Res<World>,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut entities: ResMut<EntityMapping<I>>,
) where
    E: Event + crate::tileset::ConcernedTileset<I, T, S> + oc_geo::region::Region,
    I: WcfgFrom<WorldTileIndex>
        + WcfgInto<WorldTileIndex>
        + WcfgInto<Xy>
        + Clone
        + Eq
        + Hash
        + Send
        + Sync
        + 'static,
    S: crate::tileset::Tileset<I, T>,
{
    let Some(w) = &w.0 else { return };
    let (Some(mod_), Some(meta), Some(static_), Some(connect), Some(tileset)) = (
        &mod_.0,
        &meta.0,
        &static_.0,
        &network.server,
        event.tileset(&world_),
    ) else {
        return;
    };
    let region = event.region();
    tracing::debug!("Spawn region {:?} tiles", region);

    let mod_ = mod_.canonical();
    let world = meta.canonical();
    let files = files::Files::new(mod_, world).into_gui(static_.clone(), connect.clone().into());
    let spriteset = tileset.spriteset(&files).display().to_string();

    let texture = asset_server.load(&spriteset);
    let layout = tileset.layout();
    let texture_atlas_layout = texture_atlas_layouts.add(layout);

    if let Some(tiles) = tileset.tiles(&world_, region) {
        for (i, tile) in tiles {
            let i: WorldTileIndex = (i.clone()).into_(w);
            let xy: Xy = i.into_(w);
            // TODO (map terrain should be checked to avoid manage missing terrain here)
            let index = tileset.index(&tile).unwrap();
            // let index = (*tileset.natures.get(&tile.nature).unwrap()) as usize;
            let x = xy.0 * w.geo_pixels_per_tile;
            let y = xy.1 * w.geo_pixels_per_tile;
            let z = tileset.z();
            let point = Vec3::new(x as f32, (y as f32).to_gui_y(w), z);

            let entity = commands
                .spawn((
                    TerrainTile(i),
                    oc_physics::update::bevy::Region(region),
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
            entities.insert(i.into_(w), entity);
        }
    }
}

// pub fn on_spawn_region_tiles(
//     region: On<SpawnRegionTiles>,
//     mut commands: Commands,
//     mod_: Res<Mod>,
//     meta: Res<Meta>,
//     static_: Res<StaticSource>,
//     network: Res<network::state::State>,
//     world_: Res<World>,
//     asset_server: Res<AssetServer>,
//     mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
//     mut entities: ResMut<EntityMapping<WorldTileIndex>>,
// ) {
//     let (Some(mod_), Some(meta), Some(static_), Some(connect), Some(terrain)) = (
//         &mod_.0,
//         &meta.0,
//         &static_.0,
//         &network.server,
//         &world_.terrain,
//     ) else {
//         return;
//     };
//     tracing::debug!("Spawn region {:?} tiles", region.0);

//     let mod_ = mod_.canonical();
//     let world = meta.canonical();
//     let files = files::Files::new(mod_, world).into_gui(static_.clone(), connect.clone().into());
//     let terrain_png = files.terrain_png().display().to_string();

//     let texture = asset_server.load(&terrain_png);
//     let layout = terrain.layout();
//     let texture_atlas_layout = texture_atlas_layouts.add(layout);

//     if let Some(tiles) = world_.tiles().get(&region) {
//         for (i, tile) in tiles {
//             let xy: Xy = (*i).into();
//             // TODO (map terrain should be checked to avoid manage missing terrain here)
//             let index = (*terrain.natures.get(&tile.nature).unwrap()) as usize;
//             let x = xy.0 * GEO_PIXELS_PER_TILE;
//             let y = xy.1 * GEO_PIXELS_PER_TILE;
//             let point = Vec3::new(x as f32, (y as f32).to_gui_y(), Z_TERRAIN_TILE);

//             let entity = commands
//                 .spawn((
//                     TerrainTile(*i),
//                     Region(region.0.into()),
//                     Sprite {
//                         image: texture.clone(),
//                         texture_atlas: Some(TextureAtlas {
//                             layout: texture_atlas_layout.clone(),
//                             index,
//                         }),
//                         ..Default::default()
//                     },
//                     Transform::from_translation(point),
//                     Anchor::TOP_LEFT,
//                 ))
//                 .id();
//             entities.insert(*i, entity);
//         }
//     }
// }

pub fn on_forgotten_region(region: On<ForgottenRegion>, mut commands: Commands) {
    commands.trigger(DespawnRegionTiles(region.0));
    commands.trigger(DespawnRegionHeights(region.0));
}

pub fn on_despawn_region_tiles(
    region: On<DespawnRegionTiles>,
    mut commands: Commands,
    tiles: Query<(&TerrainTile, Entity, &oc_physics::update::bevy::Region)>,
    mut entities: ResMut<EntityMapping<WorldTileIndex>>,
) {
    tracing::debug!("Despawn region {:?} tiles", region.0);

    for (tile, entity, region_) in tiles {
        if region_.0 == region.0 {
            entities.remove(&tile.0);
            commands.entity(entity).despawn();
        }
    }
}

pub fn tile_under_cursor(
    w: Res<Wcfg>,
    mut state: ResMut<State>,
    window_: Single<&Window>,
    camera_: Single<(&Camera, &GlobalTransform)>,
    entities: Res<EntityMapping<WorldTileIndex>>,
    mut tiles: Query<(&TerrainTile, &mut Sprite)>,
) {
    let Some(w) = &w.0 else { return };
    let (camera, transform) = *camera_;
    if let Some(cursor) = window_.cursor_position() {
        if let Ok(cursor) = camera.viewport_to_world_2d(transform, cursor) {
            let point = Vec2::new(cursor.x, cursor.y.to_gui_y(w));
            let tile: TileXy = [point.x, point.y].into_(w);
            let current: WorldTileIndex = tile.into_(w);

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

                        state.tile = Some(current);
                    }
                }
                None => state.tile = Some(current),
            }
        };
    };
}
