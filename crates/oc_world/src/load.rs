use std::{
    io,
    path::PathBuf,
    sync::{Arc, atomic::AtomicU32},
};

use derive_more::Constructor;
use image::imageops::FilterType;
use oc_geo::region::WorldRegionIndex;
use oc_mod::Mod;
use oc_projectile::NextProjectileId;
use oc_root::{
    MINIMAP_HEIGHT_PIXELS, MINIMAP_WIDTH_PIXELS, REGIONS_COUNT, WORLD_HEIGHT_PIXELS,
    WORLD_WIDTH_PIXELS, files, ids::Ids,
};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use thiserror::Error;

use crate::{
    World,
    cache::{self, CacheRegionBackgroundError},
    meta::{self, Meta},
    snapshot::Snapshot,
};

#[derive(Debug, Constructor)]
pub struct WorldLoader {
    mod_: Mod,
    world: PathBuf,
    cache: PathBuf,
}

impl WorldLoader {
    pub fn load(&self, ids: &Ids, snapshot: Snapshot) -> Result<World, Error> {
        tracing::info!("Check world {}", self.world.display());
        self.check()?;

        tracing::info!("Load world meta {}", self.world.meta().display());
        let meta = Meta::from_file(&self.world.meta()).map_err(MetaError::Load)?;

        // TODO: centralize caching at server startup
        self.cache(&meta)?;

        let mod_ = self.mod_.clone();
        let tiles = snapshot.tiles;
        let individuals = snapshot.individuals;
        let projectiles = snapshot
            .projectiles
            .into_iter()
            .map(|projectile| (ids.next_projectile_id(), projectile))
            .collect();

        Ok(World::new(mod_, meta, tiles, individuals, projectiles))
    }

    // TODO: add checks
    fn check(&self) -> Result<(), Error> {
        self.check_meta().map_err(Error::Meta)?;
        self.check_background()
            .map_err(|e| Error::Background(self.world.background(), e))?;

        Ok(())
    }

    fn check_background(&self) -> Result<(), BackgroundError> {
        if !self.world.background().exists() {
            return Err(BackgroundError::NotFound);
        }

        if !self.world.background().is_file() {
            return Err(BackgroundError::NotAFile);
        }

        let (width, height) = image::image_dimensions(self.world.background())?;
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
        if !self.world.meta().exists() {
            return Err(MetaError::NotFound);
        }

        if !self.world.meta().is_file() {
            return Err(MetaError::NotAFile);
        }

        Ok(())
    }

    // TODO: centralize caching at server startup
    fn cache(&self, meta: &Meta) -> Result<(), CacheError> {
        let files = files::Files::new("".to_string(), meta.canonical());
        let files = files.into_server(self.cache.clone());
        let world = files.world();
        let archive = files.world_archive();
        let minimap = files.minimap();

        std::fs::create_dir_all(&world).unwrap(); // TODO
        let image = image::open(self.world.background())?;

        match minimap.exists() {
            true => {}
            false => {
                // TODO: size in config/args ?
                let width = MINIMAP_WIDTH_PIXELS as f32;
                let height = MINIMAP_HEIGHT_PIXELS as f32;
                tracing::info!(
                    "Prepare cache for minimap {} ({}x{})",
                    minimap.display(),
                    width,
                    height
                );
                let minimap_ = image::imageops::resize(
                    &image,
                    width as u32,
                    height as u32,
                    FilterType::Gaussian,
                );
                minimap_.save(minimap)?;
            }
        }

        let counter = Arc::new(AtomicU32::new(0));
        tracing::info!("Prepare cache for regions");

        (0..REGIONS_COUNT)
            .into_par_iter()
            .map(|i| {
                let i = WorldRegionIndex(i as u64);
                let region = files.region(i.0);

                match region.exists() {
                    true => Ok(()),
                    false => {
                        counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        cache::cache_region_background(&region, &image, i)
                    }
                }
            })
            .collect::<Result<Vec<()>, CacheRegionBackgroundError>>()?;

        let cached = counter.load(std::sync::atomic::Ordering::Relaxed);
        let already = REGIONS_COUNT - cached as usize;
        tracing::info!("{} cached, {} already cached", cached, already);

        if !std::fs::exists(&archive)? {
            tracing::info!("Caching {} to {}", &self.world.display(), archive.display());

            let file = std::fs::File::create(&archive)?;
            let encoder = flate2::write::GzEncoder::new(file, flate2::Compression::default());
            let mut builder = tar::Builder::new(encoder);

            builder.append_dir_all(&meta.name, &self.world)?;
            builder.finish()?;
            tracing::info!("Finished cache for world ({})", archive.display());
        }

        Ok(())
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
    Load(#[from] meta::LoadError),
}

#[derive(Debug, Error)]
pub enum CacheError {
    #[error("Io: {0}")]
    Io(#[from] std::io::Error),
    #[error("Image: {0}")]
    Image(#[from] image::error::ImageError),
    #[error("Region background: {0}")]
    RegionBackground(#[from] CacheRegionBackgroundError),
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Background error ({0}): {1}")]
    Background(PathBuf, BackgroundError),
    #[error("Cache error: {0}")]
    Cache(#[from] CacheError),
    #[error("Meta error: {0}")]
    Meta(#[from] MetaError),
}

pub trait WorldPath {
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
