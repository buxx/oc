use std::{io, path::PathBuf};

use derive_more::Constructor;
use oc_geo::{
    region::RegionXy,
    tile::{TileXy, WorldTileIndex},
};
use oc_individual::{Individual, behavior::Behavior};
use oc_root::{
    GEO_PIXELS_PER_TILE, INDIVIDUALS_COUNT, TILES_COUNT, WORLD_HEIGHT_PIXELS, WORLD_WIDTH_PIXELS,
};
use thiserror::Error;

use crate::{World, meta::Meta, tile::Tile};

#[derive(Debug, Constructor)]
pub struct WorldLoader {
    world_path: PathBuf,
    cache_path: PathBuf,
}

impl WorldLoader {
    pub fn load(&self) -> Result<World, Error> {
        self.check()?;
        let meta = self.meta()?;

        self.cache(&meta)?;

        let tiles = vec![Tile::ShortGrass; TILES_COUNT];
        let individuals = hack_individuals();
        let world = World::new(meta, tiles, individuals);

        Ok(world)
    }

    fn check(&self) -> Result<(), Error> {
        self.check_meta().map_err(|e| Error::Meta(e))?;
        self.check_background()
            .map_err(|e| Error::Background(self.world_path.background(), e))?;

        Ok(())
    }

    fn check_background(&self) -> Result<(), BackgroundError> {
        if !self.world_path.background().exists() {
            return Err(BackgroundError::NotFound);
        }

        if !self.world_path.background().is_file() {
            return Err(BackgroundError::NotAFile);
        }

        let (width, height) = image::image_dimensions(&self.world_path.background())?;
        if width != WORLD_WIDTH_PIXELS as u32 || height != WORLD_HEIGHT_PIXELS as u32 {
            return Err(BackgroundError::Dimensions(
                width,
                height,
                WORLD_WIDTH_PIXELS as u32,
                WORLD_HEIGHT_PIXELS as u32,
            ));
        }

        Ok(())
    }

    fn check_meta(&self) -> Result<(), MetaError> {
        if !self.world_path.meta().exists() {
            return Err(MetaError::NotFound);
        }

        if !self.world_path.meta().is_file() {
            return Err(MetaError::NotAFile);
        }

        Ok(())
    }

    fn meta(&self) -> Result<Meta, MetaError> {
        todo!()
    }

    fn cache(&self, meta: &Meta) -> Result<(), CacheError> {
        // FIXME BS NOW: pour chaque region, get path normalisé region png
        // si on a pas, on decoupe. le tout dans threads
        todo!()
    }
}

#[derive(Debug, Error)]
pub enum BackgroundError {
    #[error("Not found")]
    NotFound,
    #[error("Is not a file")]
    NotAFile,
    #[error("Io: {0}")]
    Io(#[from] io::Error),
    #[error("Image: {0}")]
    Image(#[from] image::error::ImageError),
    #[error("Dimensions: current {0}x{1}, required: {2}x{3}")]
    Dimensions(u32, u32, u32, u32),
}

#[derive(Debug, Error)]
pub enum MetaError {
    #[error("Not found")]
    NotFound,
    #[error("Is not a file")]
    NotAFile,
    #[error("Io: {0}")]
    Io(#[from] io::Error),
}

#[derive(Debug, Error)]
pub enum CacheError {}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Background error ({0}): {1}")]
    Background(PathBuf, BackgroundError),
    #[error("Cache error: {0}")]
    Cache(#[from] CacheError),
    #[error("Meta error: {0}")]
    Meta(#[from] MetaError),
}

fn hack_individuals() -> Vec<Individual> {
    (0..INDIVIDUALS_COUNT)
        .map(|i| {
            let xy = TileXy::from(WorldTileIndex(i));
            let position = [
                ((xy.0.0 * GEO_PIXELS_PER_TILE) + GEO_PIXELS_PER_TILE / 2) as f32,
                ((xy.0.1 * GEO_PIXELS_PER_TILE) + GEO_PIXELS_PER_TILE / 2) as f32,
            ];
            let region: RegionXy = xy.into();
            Individual::new(position, xy, region, Behavior::Idle, vec![])
        })
        .collect()
}

trait WorldPath {
    fn background(&self) -> PathBuf;
    fn meta(&self) -> PathBuf;
}

impl WorldPath for PathBuf {
    fn background(&self) -> PathBuf {
        self.join("background.png")
    }

    fn meta(&self) -> PathBuf {
        self.join("meta.toml")
    }
}
