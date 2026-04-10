use std::{ops::Deref, path::PathBuf};

#[cfg(feature = "bevy")]
use bevy::image::TextureAtlasLayout;
#[cfg(feature = "bevy")]
use bevy::math::UVec2;
#[cfg(feature = "bevy")]
use oc_root::GEO_PIXELS_PER_TILE;
use rustc_hash::FxHashMap;
use strum::IntoEnumIterator;
use tiled::TileId;

use crate::tile::Nature;

#[derive(Debug)]
pub struct Terrain {
    pub raw: tiled::Tileset,
    pub natures: FxHashMap<Nature, TileId>,
}

impl Terrain {
    pub fn load(path: &PathBuf) -> Result<Self, Error> {
        let raw = tiled::Loader::new().load_tsx_tileset(path)?;
        let natures = oc_utils::tileset::extract(Nature::iter(), &raw);
        let natures = natures.map_err(|e| Error::ExtractTiles(path.clone(), e))?;
        let natures = natures.into_iter().collect();
        Ok(Self { raw, natures })
    }

    #[cfg(feature = "bevy")]
    pub fn layout(&self) -> TextureAtlasLayout {
        TextureAtlasLayout::from_grid(
            UVec2::splat(GEO_PIXELS_PER_TILE as u32),
            self.columns(),
            self.rows(),
            None,
            None,
        )
    }

    pub fn columns(&self) -> u32 {
        self.raw.columns
    }

    pub fn rows(&self) -> u32 {
        self.tilecount / self.columns()
    }
}

impl Deref for Terrain {
    type Target = tiled::Tileset;

    fn deref(&self) -> &Self::Target {
        &self.raw
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Error during load of {0}")]
    Load(#[from] tiled::Error),
    #[error("Error during extract tiles from {0}: {1}")]
    ExtractTiles(PathBuf, String),
}
