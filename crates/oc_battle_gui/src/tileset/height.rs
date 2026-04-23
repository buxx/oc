use std::path::PathBuf;

use bevy::{image::TextureAtlasLayout, math::UVec2};
use oc_geo::{region::WorldRegionIndex, tile::WorldHeightIndex};
use oc_root::{GEO_PIXELS_PER_TILE, files::FilesAsGui};
use oc_world::terrain::Terrain;
use rustc_hash::FxHashMap;

use crate::{ingame::draw::Z_TERRAIN_TILE, world::World};

impl super::Tileset<WorldHeightIndex, f32> for Terrain {
    fn layout(&self) -> TextureAtlasLayout {
        TextureAtlasLayout::from_grid(
            UVec2::splat(GEO_PIXELS_PER_TILE as u32),
            10, // FIXME: data from real terrain txs
            10, // FIXME: data from real terrain txs
            None,
            None,
        )
    }

    fn spriteset(&self, files: &FilesAsGui) -> PathBuf {
        files.height_png()
    }

    fn z(&self) -> f32 {
        Z_TERRAIN_TILE
    }

    fn tiles<'a>(
        &self,
        world: &'a World,
        region: WorldRegionIndex,
    ) -> Option<&'a FxHashMap<WorldHeightIndex, f32>> {
        world.heights().get(&region)
    }

    fn index(&self, height: &f32) -> Option<usize> {
        Some((*height) as usize)
    }
}
