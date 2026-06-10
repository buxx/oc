#[cfg(feature = "bevy")]
use bevy::prelude::*;

use rkyv::{Archive, Deserialize, Serialize};

use crate::physics::Meters;

pub mod end;
pub mod files;
pub mod ids;
pub mod material;
pub mod opacity;
pub mod physics;
pub mod side;
pub mod static_;
pub mod utils;
pub mod y;

#[derive(Debug, Clone, Archive, Deserialize, Serialize, PartialEq)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct WorldConfig {
    pub world_width: u64,
    pub world_height: u64,
    pub region_width: u64,
    pub region_height: u64,
    pub tiles_count: u64,
    pub regions_count: u64,
    pub regions_width: u64,
    pub regions_height: u64,
    pub world_width_pixels: u64,
    pub world_height_pixels: u64,
    pub region_width_pixels: u64,
    pub region_height_pixels: u64,
    pub individual_tick_interval_us: u64,
    pub physics_tick_per_seconds: u64,
    pub physics_tick_interval_us: u64,
    pub physics_coeff_per_tick: f32,
    pub geo_pixels_per_meters: f32,
    pub geo_pixels_per_tile: u64,
    pub geo_bresenham_precision: f32,
    pub geo_bresenham_step: u64,
    pub geo_meters_per_z: Meters,
    pub geo_lov_step: u64,
    pub minimap_width_pixels: u64,
    pub minimap_height_pixels: u64,
}

impl WorldConfig {
    pub fn new(world_width: u64, world_height: u64, geo_meters_per_z: Meters) -> Self {
        let region_width = 1000.min(world_width);
        let region_height = 1000.min(world_height);
        let individual_tick_interval_us: u64 = 1_000_000 / 1;
        let physics_tick_per_seconds: u64 = 1; // FIXME BS NOW: si on baisse cette valeur on ne devrait pas voir les choses aller plus doucement !
        let physics_tick_interval_us: u64 = 1_000_000 / physics_tick_per_seconds;
        let physics_coeff_per_tick: f32 = 1. / physics_tick_per_seconds as f32;
        let geo_pixels_per_meters: f32 = 5.;
        let geo_pixels_per_tile: u64 = geo_pixels_per_meters as u64;
        let geo_bresenham_precision: f32 = 100.;
        let geo_bresenham_step: u64 = 250;
        let geo_lov_step: u64 = 0; // TODO: don't work if not 0 (pointillés)

        let tiles_count = world_width * world_height;
        let regions_count = tiles_count / (region_width * region_height);
        let regions_width = world_width / region_width;
        let regions_height = world_height / region_height;
        let world_width_pixels = world_width * geo_pixels_per_tile;
        let world_height_pixels = world_height * geo_pixels_per_tile;
        let region_width_pixels = region_width * geo_pixels_per_tile;
        let region_height_pixels = region_height * geo_pixels_per_tile;
        let minimap_width_pixels: u64 = 2048;
        let minimap_height_pixels: u64 = 2048;

        Self {
            world_width,
            world_height,
            region_width,
            region_height,
            tiles_count,
            regions_count,
            regions_width,
            regions_height,
            world_width_pixels,
            world_height_pixels,
            region_width_pixels,
            region_height_pixels,
            individual_tick_interval_us,
            physics_tick_per_seconds,
            physics_tick_interval_us,
            physics_coeff_per_tick,
            geo_pixels_per_meters,
            geo_pixels_per_tile,
            geo_bresenham_precision,
            geo_bresenham_step,
            geo_meters_per_z,
            geo_lov_step,
            minimap_width_pixels,
            minimap_height_pixels,
        }
    }

    pub fn region_width(mut self, value: u64) -> Self {
        self.region_width = value;
        self.regions_count = self.tiles_count / (self.region_width * self.region_height);
        self.regions_width = self.world_width / self.region_width;
        self.region_width_pixels = self.region_width * self.geo_pixels_per_tile;
        self
    }

    pub fn region_height(mut self, value: u64) -> Self {
        self.region_height = value;
        self.regions_count = self.tiles_count / (self.region_width * self.region_height);
        self.regions_height = self.world_height / self.region_height;
        self.region_height_pixels = self.region_height * self.geo_pixels_per_tile;
        self
    }

    pub fn physics_coeff_per_tick(mut self, value: f32) -> Self {
        self.physics_coeff_per_tick = value;
        self
    }

    pub fn geo_bresenham_precision(mut self, value: f32) -> Self {
        self.geo_bresenham_precision = value;
        self
    }

    pub fn geo_bresenham_step(mut self, value: u64) -> Self {
        self.geo_bresenham_step = value;
        self
    }

    pub fn geo_pixels_per_meters(mut self, value: f32) -> Self {
        self.geo_pixels_per_meters = value;
        self
    }

    pub fn geo_pixels_per_tile(mut self, value: u64) -> Self {
        self.geo_pixels_per_tile = value;
        self
    }

    pub fn geo_lov_step(mut self, value: u64) -> Self {
        self.geo_lov_step = value;
        self
    }
}

pub trait Client: Clone + std::hash::Hash + Eq + std::fmt::Debug + Send + Sync + 'static {}
impl<T: Clone + std::hash::Hash + Eq + std::fmt::Debug + Send + Sync + 'static> Client for T {}

pub trait WcfgFrom<T>: Sized {
    fn from_(value: T, w: &WorldConfig) -> Self;
}

pub trait WcfgInto<T>: Sized {
    fn into_(self, w: &WorldConfig) -> T;
}

impl<T, U> WcfgInto<U> for T
where
    U: WcfgFrom<T>,
{
    fn into_(self, w: &WorldConfig) -> U {
        U::from_(self, w)
    }
}

#[cfg(feature = "bevy")]
#[derive(Debug, Resource, Deref, Default)]
pub struct Wcfg(pub Option<WorldConfig>);
