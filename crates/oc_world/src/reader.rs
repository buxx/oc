use std::{collections::HashMap, path::PathBuf, str::FromStr, sync::Arc};

use crate::{
    spawn::{ParseOriginDirectionError, SpawnZoneName},
    tile::{Nature, NatureError, Tile},
};
use glam::Vec2;
use oc_geo::tile::{TileXy, WorldTileIndex};
use oc_utils::d2::Xy;
use tiled::{
    FiniteTileLayer, Image, ImageLayer, Layer, LayerType, Loader, Map as TiledMap, ObjectLayer,
    TileLayer, Tileset,
};

use crate::flag::{Flag, FlagName};

use crate::{
    decor::{Decor, DecorTile},
    interior::Interior,
    map::Map,
    spawn::SpawnZone,
};

type DecorTilesets = (Vec<Arc<Tileset>>, HashMap<usize, usize>);

const BACKGROUND_IMAGE_LAYER_NAME: &str = "background_image";
const INTERIORS_IMAGE_LAYER_NAME: &str = "interiors_image";
const INTERIORS_ZONES_LAYER_NAME: &str = "interiors_zones";
const SPAWN_ZONES_LAYER_NAME: &str = "spawn_zones";
const FLAGS_LAYER_NAME: &str = "flags";
const DECOR_LAYER_NAME: &str = "decor";
const TERRAIN_LAYER_NAME: &str = "terrain";
const HEIGHT_LAYER_NAME: &str = "height";
const TERRAIN_TILESET_NAME: &str = "terrain";
const HEIGHT_TILESET_NAME: &str = "height";
const TILE_ID_PROPERTY_KEY: &str = "ID";

#[derive(Debug, thiserror::Error)]
pub enum MapReaderError {
    #[error("Map not found: {0}")]
    MapNotFound(String),
    #[error("Layer not found: {0}")]
    LayerNotFound(String),
    #[error("Invalid layer: {0}")]
    InvalidLayer(String),
    #[error("Tile set not found: {0}")]
    TileSetNotFound(String),
    #[error("Invalid tile set: {0}")]
    InvalidTileSet(String),
    #[error("Tile error: {0}")]
    TileError(String),
    #[error("Terrain tile error: {0}")]
    TerrainTileError(NatureError),
}

impl From<NatureError> for MapReaderError {
    fn from(error: NatureError) -> Self {
        Self::TerrainTileError(error)
    }
}

impl From<ParseOriginDirectionError> for MapReaderError {
    fn from(value: ParseOriginDirectionError) -> Self {
        Self::InvalidLayer(format!("Invalid origin direction : '{}'", value))
    }
}

pub struct MapReader {
    folder: PathBuf,
    map: TiledMap,
}

impl MapReader {
    pub fn new(path: &PathBuf) -> Result<Self, MapReaderError> {
        let mut loader = Loader::new();

        let map = match loader.load_tmx_map(&path.join("world.tmx")) {
            Ok(map) => map,
            Err(error) => {
                return Result::Err(MapReaderError::MapNotFound(format!(
                    "Failed to load map {} : {}",
                    path.display(),
                    error
                )));
            }
        };

        Ok(Self {
            folder: path.clone(),
            map,
        })
    }

    fn layer(&self, name: &str) -> Result<Layer<'_>, MapReaderError> {
        match self
            .map
            .layers()
            .filter(|layer| layer.name == name)
            .collect::<Vec<Layer>>()
            .first()
        {
            Some(layer) => Ok(*layer),
            None => Result::Err(MapReaderError::LayerNotFound(format!(
                "Failed to find layer '{}' in map",
                name,
            ))),
        }
    }

    fn background_image_layer(&self) -> Result<ImageLayer<'_>, MapReaderError> {
        match self.layer(BACKGROUND_IMAGE_LAYER_NAME)?.layer_type() {
            LayerType::ImageLayer(layer) => Ok(layer),
            _ => Result::Err(MapReaderError::InvalidLayer(format!(
                "Layer '{}' in map is not an image layer",
                BACKGROUND_IMAGE_LAYER_NAME,
            ))),
        }
    }

    fn background_image(&self) -> Result<Image, MapReaderError> {
        match &self.background_image_layer()?.image {
            Some(image) => Ok(image.clone()),
            None => Result::Err(MapReaderError::InvalidLayer(format!(
                "Layer '{}' in ma must contains image",
                BACKGROUND_IMAGE_LAYER_NAME,
            ))),
        }
    }

    fn interiors_image_layer(&self) -> Result<ImageLayer<'_>, MapReaderError> {
        match self.layer(INTERIORS_IMAGE_LAYER_NAME)?.layer_type() {
            LayerType::ImageLayer(layer) => Ok(layer),
            _ => Result::Err(MapReaderError::InvalidLayer(format!(
                "Layer '{}' in map is not an image layer",
                INTERIORS_IMAGE_LAYER_NAME,
            ))),
        }
    }

    fn interiors_image(&self) -> Result<Image, MapReaderError> {
        match &self.interiors_image_layer()?.image {
            Some(image) => Ok(image.clone()),
            None => Result::Err(MapReaderError::InvalidLayer(format!(
                "Layer '{}' in map must contains image",
                INTERIORS_IMAGE_LAYER_NAME,
            ))),
        }
    }

    fn interiors_zones_layer(&self) -> Result<ObjectLayer<'_>, MapReaderError> {
        match self.layer(INTERIORS_ZONES_LAYER_NAME)?.layer_type() {
            LayerType::ObjectLayer(layer) => Ok(layer),
            _ => Result::Err(MapReaderError::InvalidLayer(format!(
                "Layer '{}' in map is not an object layer",
                INTERIORS_ZONES_LAYER_NAME,
            ))),
        }
    }

    fn spawn_zones_layer(&self) -> Result<ObjectLayer<'_>, MapReaderError> {
        match self.layer(SPAWN_ZONES_LAYER_NAME)?.layer_type() {
            LayerType::ObjectLayer(layer) => Ok(layer),
            _ => Result::Err(MapReaderError::InvalidLayer(format!(
                "Layer '{}' in map is not an object layer",
                SPAWN_ZONES_LAYER_NAME,
            ))),
        }
    }

    fn flags_layer(&self) -> Result<ObjectLayer<'_>, MapReaderError> {
        match self.layer(FLAGS_LAYER_NAME)?.layer_type() {
            LayerType::ObjectLayer(layer) => Ok(layer),
            _ => Result::Err(MapReaderError::InvalidLayer(format!(
                "Layer '{}' in map is not an object layer",
                FLAGS_LAYER_NAME,
            ))),
        }
    }

    fn interiors(&self) -> Result<Vec<Interior>, MapReaderError> {
        let interiors_image = self.interiors_image()?;
        let mut interiors = vec![];

        for object in self.interiors_zones_layer()?.objects() {
            interiors.push(match object.shape {
                tiled::ObjectShape::Rect { width, height } => Interior::new(
                    object.x,
                    object.y,
                    width,
                    height,
                    interiors_image.width as f32,
                    interiors_image.height as f32,
                ),
                _ => {
                    return Result::Err(MapReaderError::InvalidLayer(format!(
                        "Layer '{}' in map contains non Rect shapes, this is not supported now",
                        INTERIORS_ZONES_LAYER_NAME,
                    )));
                }
            })
        }

        Ok(interiors)
    }

    fn spawn_zones(&self) -> Result<Vec<SpawnZone>, MapReaderError> {
        let background_image = self.background_image()?;
        let mut spawn_zones = vec![];

        for object in self.spawn_zones_layer()?.objects() {
            let spawn_zone_name = SpawnZoneName::from_str(&object.name)?;
            if !spawn_zone_name.allowed_for_zone_object() {
                return Err(MapReaderError::InvalidLayer(format!(
                    "Spawn zone name is not allowed : '{}'",
                    &object.name
                )));
            }

            spawn_zones.push(match object.shape {
                tiled::ObjectShape::Rect { width, height } => SpawnZone::new(
                    spawn_zone_name,
                    object.x,
                    object.y,
                    width,
                    height,
                    background_image.width as f32,
                    background_image.height as f32,
                ),
                _ => {
                    return Result::Err(MapReaderError::InvalidLayer(format!(
                        "Layer '{}' in map contains non Rect shapes, this is not supported now",
                        SPAWN_ZONES_LAYER_NAME,
                    )));
                }
            })
        }

        Ok(spawn_zones)
    }

    fn flags(&self) -> Result<Vec<Flag>, MapReaderError> {
        let mut flags = vec![];

        for object in self.flags_layer()?.objects() {
            let flag_name = FlagName(object.name.clone());

            flags.push(match object.shape {
                tiled::ObjectShape::Rect { width, height } => {
                    Flag::new(flag_name, object.x, object.y, width, height)
                }
                _ => {
                    return Result::Err(MapReaderError::InvalidLayer(format!(
                        "Layer '{}' in map contains non Rect shapes, this is not supported now",
                        FLAGS_LAYER_NAME,
                    )));
                }
            })
        }

        Ok(flags)
    }

    fn terrain_layer(&self) -> Result<FiniteTileLayer<'_>, MapReaderError> {
        match self.layer(TERRAIN_LAYER_NAME)?.layer_type() {
            LayerType::TileLayer(layer) => match layer {
                TileLayer::Finite(layer) => Ok(layer),
                TileLayer::Infinite(_) => Result::Err(MapReaderError::InvalidLayer(format!(
                    "Layer '{}' in map is an infinite tile layer, but on finite layer is supported",
                    TERRAIN_LAYER_NAME,
                ))),
            },
            _ => Result::Err(MapReaderError::InvalidLayer(format!(
                "Layer '{}' in map is not an tile layer",
                TERRAIN_LAYER_NAME,
            ))),
        }
    }

    fn height_layer(&self) -> Result<FiniteTileLayer<'_>, MapReaderError> {
        match self.layer(HEIGHT_LAYER_NAME)?.layer_type() {
            LayerType::TileLayer(layer) => match layer {
                TileLayer::Finite(layer) => Ok(layer),
                TileLayer::Infinite(_) => Result::Err(MapReaderError::InvalidLayer(format!(
                    "Layer '{}' in map is an infinite tile layer, but on finite layer is supported",
                    HEIGHT_LAYER_NAME,
                ))),
            },
            _ => Result::Err(MapReaderError::InvalidLayer(format!(
                "Layer '{}' in map is not an tile layer",
                HEIGHT_LAYER_NAME,
            ))),
        }
    }

    fn decor_layer(&self) -> Result<(Layer<'_>, FiniteTileLayer<'_>), MapReaderError> {
        let decor_layer = self.layer(DECOR_LAYER_NAME)?;
        match decor_layer.layer_type() {
            LayerType::TileLayer(layer) => match layer {
                TileLayer::Finite(layer_) => Ok((decor_layer, layer_)),
                TileLayer::Infinite(_) => Result::Err(MapReaderError::InvalidLayer(format!(
                    "Layer '{}' in map is an infinite tile layer, but on finite layer is supported",
                    DECOR_LAYER_NAME,
                ))),
            },
            _ => Result::Err(MapReaderError::InvalidLayer(format!(
                "Layer '{}' in map is not an tile layer",
                DECOR_LAYER_NAME,
            ))),
        }
    }

    fn width(&self) -> Result<u32, MapReaderError> {
        Ok(self.terrain_layer()?.width())
    }

    fn height(&self) -> Result<u32, MapReaderError> {
        Ok(self.terrain_layer()?.height())
    }

    fn terrain_tileset(&self) -> Result<&Arc<Tileset>, MapReaderError> {
        match self
            .map
            .tilesets()
            .iter()
            .filter(|tileset| tileset.name == TERRAIN_TILESET_NAME)
            .collect::<Vec<&Arc<Tileset>>>()
            .first()
        {
            Some(tileset) => Ok(tileset),
            None => Result::Err(MapReaderError::TileSetNotFound(format!(
                "Can't found terrain tileset in map must exist but is not found",
            ))),
        }
    }

    fn height_tileset(&self) -> Result<&Arc<Tileset>, MapReaderError> {
        match self
            .map
            .tilesets()
            .iter()
            .filter(|tileset| tileset.name == HEIGHT_TILESET_NAME)
            .collect::<Vec<&Arc<Tileset>>>()
            .first()
        {
            Some(tileset) => Ok(tileset),
            None => Result::Err(MapReaderError::TileSetNotFound(format!(
                "Can't found height tileset in map must exist but is not found",
            ))),
        }
    }

    fn terrain_image(&self) -> Result<Image, MapReaderError> {
        match &self.terrain_tileset()?.image {
            Some(image) => Ok(image.clone()),
            None => Result::Err(MapReaderError::InvalidTileSet(format!(
                "Terrain tileset in map should contains image",
            ))),
        }
    }

    fn height_image(&self) -> Result<Image, MapReaderError> {
        match &self.height_tileset()?.image {
            Some(image) => Ok(image.clone()),
            None => Result::Err(MapReaderError::InvalidTileSet(format!(
                "Height tileset in map should contains image",
            ))),
        }
    }

    pub fn tiles(&self) -> Result<Vec<Tile>, MapReaderError> {
        let terrain_layer = self.terrain_layer()?;
        let height_layer = self.height_layer()?;
        let terrain_tileset = self.terrain_tileset()?;
        let height_tileset = self.height_tileset()?;
        let mut tiles = vec![];

        for y in 0..terrain_layer.height() {
            for x in 0..terrain_layer.width() {
                let terrain_layer_tile_data = match terrain_layer.get_tile_data(x as i32, y as i32)
                {
                    Some(data) => data,
                    None => {
                        return Result::Err(MapReaderError::TileError(format!(
                            "Tile at '{}'x'{}' in terrain layer in map must exist but is not found",
                            x, y,
                        )));
                    }
                };
                let height_layer_tile_data = match height_layer.get_tile_data(x as i32, y as i32) {
                    Some(data) => data,
                    None => {
                        return Result::Err(MapReaderError::TileError(format!(
                            "Tile at '{}'x'{}' in height layer in map must exist but is not found",
                            x, y,
                        )));
                    }
                };
                let terrain_tile_data = match terrain_tileset.get_tile(terrain_layer_tile_data.id())
                {
                    Some(tile) => tile.clone(),
                    None => {
                        return Result::Err(MapReaderError::TileError(format!(
                            "Tile '{}' in terrain layer in map is not found in tilesets",
                            terrain_layer_tile_data.id(),
                        )));
                    }
                };

                let id = match terrain_tile_data.properties.get(TILE_ID_PROPERTY_KEY) {
                    Some(id) => match id {
                        tiled::PropertyValue::StringValue(id) => id,
                        _ => {
                            return Result::Err(MapReaderError::TileError(format!(
                                "Tile '{}' in terrain layer in map should contains {} string property but it is not",
                                terrain_layer_tile_data.id(),
                                TILE_ID_PROPERTY_KEY,
                            )));
                        }
                    },
                    None => {
                        return Result::Err(MapReaderError::TileError(format!(
                            "Tile '{}' in terrain layer in map should contains {} property",
                            terrain_layer_tile_data.id(),
                            TILE_ID_PROPERTY_KEY,
                        )));
                    }
                };

                let z = height_layer_tile_data.id() as u8;
                let tile_id = terrain_layer_tile_data.id();
                let tile_y = tile_id / terrain_tileset.columns;
                let tile_x = tile_id - (tile_y * terrain_tileset.columns);
                let nature = Nature::from_str(id)?;
                let i: WorldTileIndex = TileXy(Xy(tile_x as u64, tile_y as u64)).into();
                let tile = Tile { i, nature, z };

                tiles.push(tile)
            }
        }

        Ok(tiles)
    }

    fn decor_tilesets(&self) -> Result<DecorTilesets, MapReaderError> {
        let (_, layer_) = self.decor_layer()?;
        let mut tileset_indexes = vec![];
        let mut tilesets = vec![];
        let mut positions = HashMap::new();

        for x in 0..layer_.width() {
            for y in 0..layer_.height() {
                if let Some(layer_tile_data) = layer_.get_tile_data(x as i32, y as i32) {
                    if !tileset_indexes.contains(&layer_tile_data.tileset_index()) {
                        tileset_indexes.push(layer_tile_data.tileset_index());
                    }
                };
            }
        }

        for (i, tileset) in self.map.tilesets().iter().enumerate() {
            if tileset_indexes.contains(&i) {
                positions.insert(i, tilesets.len());
                tilesets.push(tileset.clone());
            }
        }

        Ok((tilesets, positions))
    }

    fn decor_images(&self) -> Result<Vec<Image>, MapReaderError> {
        let mut images = vec![];
        let (tilesets, _) = self.decor_tilesets()?;

        for tileset in tilesets {
            match &tileset.image {
                Some(image) => images.push(image.clone()),
                None => {
                    return Result::Err(MapReaderError::InvalidTileSet(format!(
                        "All decor tileset in map must contais image",
                    )));
                }
            };
        }

        Ok(images)
    }

    fn decor(&self) -> Result<Decor, MapReaderError> {
        let (decor_layer, decor_layer_) = self.decor_layer()?;
        let (_, tilesets_positions) = self.decor_tilesets()?;
        let images = self.decor_images()?;
        let image_paths = images
            .iter()
            .map(|image| self.folder.join(&image.source))
            .collect();

        let mut tiles = vec![];

        for x in 0..decor_layer_.width() {
            for y in 0..decor_layer_.height() {
                if let Some(layer_tile_data) = decor_layer_.get_tile_data(x as i32, y as i32) {
                    let tileset = self.map.tilesets()[layer_tile_data.tileset_index()].clone();

                    let decor_tileset_position = *tilesets_positions
                        .get(&layer_tile_data.tileset_index())
                        .expect("Positions must are consistent");
                    let image = images
                        .get(decor_tileset_position)
                        .expect("Positions must are consistent");
                    let tile_width = tileset.tile_width;
                    let tile_height = tileset.tile_height;
                    let relative_tile_width = tile_width as f32 / image.width as f32;
                    let relative_tile_height = tile_height as f32 / image.height as f32;

                    let tile_id = layer_tile_data.id();
                    let tile_y = tile_id / tileset.columns;
                    let tile_x = tile_id - (tile_y * tileset.columns);

                    let terrain_tile = DecorTile::new(
                        decor_tileset_position,
                        tile_width,
                        tile_height,
                        relative_tile_width,
                        relative_tile_height,
                        x,
                        y,
                        tile_x,
                        tile_y,
                    );

                    tiles.push(terrain_tile);
                };
            }
        }

        let offset = Vec2::new(-decor_layer.offset_x, -decor_layer.offset_y);
        Ok(Decor::new(image_paths, tiles, offset))
    }

    pub fn build(&self) -> Result<Map, MapReaderError> {
        let background_image_path = self.folder.join(&self.background_image()?.source);
        let interiors_image_path = self.folder.join(&self.interiors_image()?.source);
        let terrain_image_path = self.folder.join(&self.terrain_image()?.source);

        let interiors = self.interiors()?;
        let spawn_zones = self.spawn_zones()?;
        let width = self.width()?;
        let height = self.height()?;
        let decor = self.decor()?;
        let flags = self.flags()?;

        Ok(Map::new(
            background_image_path,
            interiors_image_path,
            terrain_image_path,
            interiors,
            spawn_zones,
            width,
            height,
            decor,
            flags,
        ))
    }
}
