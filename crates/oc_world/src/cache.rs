use std::path::PathBuf;

use image::DynamicImage;
use oc_geo::region::{RegionXy, WorldRegionIndex};
use oc_root::{REGION_HEIGHT_PIXELS, REGION_WIDTH_PIXELS};
use thiserror::Error;

pub fn cache_region_background(
    output: &PathBuf,
    image: &DynamicImage,
    i: WorldRegionIndex,
) -> Result<(), CacheRegionBackgroundError> {
    let xy: RegionXy = i.into();
    let x = xy.0.0 * REGION_WIDTH_PIXELS;
    let y = xy.0.1 * REGION_HEIGHT_PIXELS;
    let width = REGION_WIDTH_PIXELS as u32;
    let height = REGION_HEIGHT_PIXELS as u32;
    let region = image.crop_imm(x as u32, y as u32, width, height);
    region.save(output)?;
    Ok(())
}

#[derive(Debug, Error)]
pub enum CacheRegionBackgroundError {
    #[error("Io: {0}")]
    Io(#[from] std::io::Error),
    #[error("Image: {0}")]
    Image(#[from] image::error::ImageError),
}
