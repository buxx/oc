#[cfg(feature = "debug")]
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

#[cfg(feature = "bevy")]
use bevy::prelude::*;

use rkyv::{Archive, Deserialize, Serialize};

use crate::{
    opacity::CumulatedOpacity,
    physics::{Meters, Seconds},
};

pub mod end;
pub mod files;
pub mod geo;
pub mod identity;
pub mod ids;
pub mod material;
pub mod opacity;
pub mod physics;
pub mod side;
pub mod static_;
pub mod utils;
pub mod y;

#[cfg(feature = "debug")]
static INACCURACY_SPREAD_RAW: AtomicU32 = AtomicU32::new(0);
#[cfg(feature = "debug")]
static INACCURACY_SPREAD_ENABLED: AtomicBool = AtomicBool::new(false);

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
    pub visibilities_tick_interval_us: u64,
    pub squad_tick_interval_us: u64,
    pub physics_tick_per_seconds: u64,
    pub physics_tick_interval_us: u64,
    pub physics_coeff_per_tick: f32,
    pub geo_pixels_per_meters: f32,
    pub geo_pixels_per_tile: u64,
    pub geo_meters_per_z: Meters,
    pub minimap_width_pixels: u64,
    pub minimap_height_pixels: u64,
    pub formation_tiles_between_positions: u64,
    pub individual_visibility_until: CumulatedOpacity,
    /// Inaccuracy start value
    pub base_inaccuracy: f32,
    /// Inaccuracy value for 100% suppressed (50% suppressed will by 50% of this value)
    pub suppress_inaccuracy: f32,
    pub standup_inaccuracy: f32,
    pub walking_inaccuracy: f32,
    pub running_inaccuracy: f32,
    pub crawling_inaccuracy: f32,
    pub prone_inaccuracy: f32,
    /// Inaccuracy value for 100% opacity (50% opacity will by 50% of this value)
    pub opacity_inaccuracy: f32,
    /// To prevent problems due to "square 3d" (when gunner is close to the edge)
    pub ignore_firsts_lov_tiles: u8,
    /// To prevent problems due to "square 3d" (when gunner is close to the edge)
    pub ignore_firsts_physics_pixels: u8,
}

impl WorldConfig {
    pub fn new(world_width: u64, world_height: u64, geo_meters_per_z: Meters) -> Self {
        let region_width = 1000.min(world_width);
        let region_height = 1000.min(world_height);
        let individual_tick_interval_us: u64 = 1_000_000 / 1;
        let visibilities_tick_interval_us: u64 = (1_000_000 as f32 / 0.2) as u64;
        let squad_tick_interval_us: u64 = (1_000_000 as f32 / 0.5) as u64;
        // FIXME: delta is computed statically here (physics_coeff_per_tick) but maybe should
        // be computed from real eslapsec time between physics iterations
        let physics_tick_per_seconds: u64 = 10;
        let physics_tick_interval_us: u64 = 1_000_000 / physics_tick_per_seconds;
        let physics_coeff_per_tick: f32 = 1. / physics_tick_per_seconds as f32;
        let geo_pixels_per_meters: f32 = 5.;
        let geo_pixels_per_tile: u64 = geo_pixels_per_meters as u64;

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

        let formation_tiles_between_positions = 2;
        let individual_visibility_until = CumulatedOpacity(0.6);
        let base_inaccuracy = 0.05;
        let suppress_inaccuracy = 2.0;
        let standup_inaccuracy = 0.1;
        let walking_inaccuracy = 1.5;
        let running_inaccuracy = 2.5;
        let crawling_inaccuracy = 1.8;
        let prone_inaccuracy = 0.0;
        let opacity_inaccuracy = 2.0;
        let ignore_firsts_lov_tiles = 2;
        let ignore_firsts_physics_pixels = 8;

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
            visibilities_tick_interval_us,
            squad_tick_interval_us,
            physics_tick_per_seconds,
            physics_tick_interval_us,
            physics_coeff_per_tick,
            geo_pixels_per_meters,
            geo_pixels_per_tile,
            geo_meters_per_z,
            minimap_width_pixels,
            minimap_height_pixels,
            formation_tiles_between_positions,
            individual_visibility_until,
            base_inaccuracy,
            suppress_inaccuracy,
            standup_inaccuracy,
            walking_inaccuracy,
            running_inaccuracy,
            crawling_inaccuracy,
            prone_inaccuracy,
            opacity_inaccuracy,
            ignore_firsts_lov_tiles,
            ignore_firsts_physics_pixels,
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

    pub fn individual_tick_interval_us(mut self, value: u64) -> Self {
        self.individual_tick_interval_us = value;
        self
    }

    pub fn physics_coeff_per_tick(mut self, value: f32) -> Self {
        self.physics_coeff_per_tick = value;
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

    pub fn formation_tiles_between_positions(mut self, value: u64) -> Self {
        self.formation_tiles_between_positions = value;
        self
    }

    pub fn visibilities_tick_each_seconds(mut self, value: f32) -> Self {
        self.visibilities_tick_interval_us = (1_000_000 as f32 / value) as u64;
        self
    }

    #[cfg(feature = "debug")]
    pub fn inaccuracy_spread() -> f32 {
        INACCURACY_SPREAD_RAW.load(Ordering::Relaxed) as f32 / 10_000.
    }

    #[cfg(not(feature = "debug"))]
    pub fn inaccuracy_spread() -> f32 {
        0.0
    }

    #[cfg(feature = "debug")]
    pub fn set_inaccuracy_spread(value: f32) {
        INACCURACY_SPREAD_RAW.store((value * 10_000.) as u32, Ordering::Relaxed);
    }

    #[cfg(not(feature = "debug"))]
    pub fn set_inaccuracy_spread(_: f32) {}

    #[cfg(feature = "debug")]
    pub fn inaccuracy_spread_enabled() -> bool {
        INACCURACY_SPREAD_ENABLED.load(Ordering::Relaxed)
    }

    #[cfg(not(feature = "debug"))]
    pub fn inaccuracy_spread_enabled() -> bool {
        false
    }

    #[cfg(feature = "debug")]
    pub fn set_inaccuracy_spread_enabled(value: bool) {
        INACCURACY_SPREAD_ENABLED.store(value, Ordering::Relaxed);
    }

    #[cfg(not(feature = "debug"))]
    pub fn set_inaccuracy_spread_enabled(_: bool) {}
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

#[derive(
    Debug,
    Clone,
    Copy,
    Archive,
    rkyv::Deserialize,
    rkyv::Serialize,
    PartialEq,
    serde::Deserialize,
    serde::Serialize,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct Suppress(u8);

impl Suppress {
    pub fn zero() -> Self {
        Self(0)
    }

    pub fn normalize(&self) -> f32 {
        self.0 as f32 / 255.0
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    Archive,
    rkyv::Deserialize,
    rkyv::Serialize,
    PartialEq,
    serde::Deserialize,
    serde::Serialize,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct U8Progress(pub u8);

impl U8Progress {
    pub fn zero() -> Self {
        Self(0)
    }

    pub fn tick(&self, interval_micros: u64, total: Seconds) -> Self {
        let total_micros = (total.0 * 1_000_000.0) as u64;
        let total_ticks = (total_micros / interval_micros).max(1); // avoid div-by-zero
        let increment = (255u32 / total_ticks as u32) as u8;
        Self(self.0.saturating_add(increment))
    }

    pub fn finished(&self) -> bool {
        self.0 == 255
    }
}

// WARN: U8Progress tests AI generated
#[cfg(test)]
mod tests {
    use super::*;

    // 2s action, 10 ticks/sec -> interval = 100_000 µs, total_ticks = 20
    // increment = 255 / 20 = 12 (integer division)
    #[test]
    fn single_tick_2s_at_10tps() {
        let p = U8Progress(0);
        let p = p.tick(100_000, Seconds(2.0));
        assert_eq!(p.0, 12);
    }

    // 1s action, 10 ticks/sec -> interval = 100_000 µs, total_ticks = 10
    // increment = 255 / 10 = 25  (this matches your original expected example)
    #[test]
    fn single_tick_1s_at_10tps() {
        let p = U8Progress(0);
        let p = p.tick(100_000, Seconds(1.0));
        assert_eq!(p.0, 25);
    }

    // Accumulation over the full duration of a 2s/10tps action.
    // 20 ticks * 12 = 240, NOT 255 -- documents the truncation issue.
    #[test]
    fn full_duration_does_not_reach_max_due_to_truncation() {
        let mut p = U8Progress(0);
        for _ in 0..20 {
            p = p.tick(100_000, Seconds(2.0));
        }
        assert_eq!(p.0, 240);
        assert!(p.0 < 255);
    }

    // Progress must never exceed 255 (u8::MAX) even with excess ticks.
    #[test]
    fn saturates_at_max() {
        let mut p = U8Progress(0);
        for _ in 0..100 {
            p = p.tick(100_000, Seconds(1.0)); // increment 25 each time
        }
        assert_eq!(p.0, 255);
    }

    // Starting near the top should saturate, not wrap around.
    #[test]
    fn saturating_add_does_not_wrap() {
        let p = U8Progress(250);
        let p = p.tick(100_000, Seconds(1.0)); // increment 25 -> would be 275
        assert_eq!(p.0, 255);
    }

    // interval longer than total duration -> total_ticks clamped to 1,
    // increment = 255 / 1 = 255 (jumps straight to max in one tick).
    #[test]
    fn interval_larger_than_total_clamped_to_one_tick() {
        let p = U8Progress(0);
        let p = p.tick(5_000_000, Seconds(1.0)); // 5s interval, 1s total
        assert_eq!(p.0, 255);
    }

    // Sanity check: zero-length action still doesn't panic (div-by-zero guarded).
    #[test]
    fn zero_duration_does_not_panic() {
        let p = U8Progress(0);
        let p = p.tick(100_000, Seconds(0.0));
        assert_eq!(p.0, 255); // total_ticks clamped to 1 -> full jump
    }

    // Progress from non-zero starting point still respects saturating add.
    #[test]
    fn tick_from_nonzero_start() {
        let p = U8Progress(100);
        let p = p.tick(100_000, Seconds(2.0)); // increment 12
        assert_eq!(p.0, 112);
    }
}
