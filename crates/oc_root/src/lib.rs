pub mod end;
pub mod files;
pub mod ids;
pub mod physics;
pub mod static_;
pub mod y;

// TODO: we should not have default value. Just set default IDE config to permet code check.
pub const WORLD_WIDTH: usize = _usize(option_env!("WORLD_WIDTH"), "1000");
pub const WORLD_HEIGHT: usize = _usize(option_env!("WORLD_HEIGHT"), "1000");
pub const REGION_WIDTH: usize = _usize(option_env!("REGION_WIDTH"), "100");
pub const REGION_HEIGHT: usize = _usize(option_env!("REGION_HEIGHT"), "100");

pub const TILES_COUNT: usize = WORLD_WIDTH * WORLD_HEIGHT;
pub const REGIONS_COUNT: usize = TILES_COUNT / (REGION_WIDTH * REGION_HEIGHT);
pub const REGIONS_WIDTH: usize = WORLD_WIDTH / REGION_WIDTH;
pub const REGIONS_HEIGHT: usize = WORLD_HEIGHT / REGION_HEIGHT;

pub const INDIVIDUAL_TICK_INTERVAL_US: u64 = 1_000_000 / 4;

pub const PHYSICS_TICK_PER_SECONDS: u64 = 10;
pub const PHYSICS_TICK_INTERVAL_US: u64 = 1_000_000 / PHYSICS_TICK_PER_SECONDS;
pub const PHYSICS_COEFF_PER_TICK: f32 = 1. / PHYSICS_TICK_PER_SECONDS as f32;

pub const GEO_PIXELS_PER_METERS: f32 = 5.;
pub const GEO_PIXELS_PER_TILE: u64 = GEO_PIXELS_PER_METERS as u64;
pub const GEO_BRESENHAM_PRECISION: f32 = 100.;
pub const GEO_BRESENHAM_STEP: usize = 250;

pub const WORLD_WIDTH_PIXELS: u64 = WORLD_WIDTH as u64 * GEO_PIXELS_PER_TILE;
pub const WORLD_HEIGHT_PIXELS: u64 = WORLD_HEIGHT as u64 * GEO_PIXELS_PER_TILE;
pub const REGION_WIDTH_PIXELS: u64 = REGION_WIDTH as u64 * GEO_PIXELS_PER_TILE;
pub const REGION_HEIGHT_PIXELS: u64 = REGION_HEIGHT as u64 * GEO_PIXELS_PER_TILE;

pub const MINIMAP_WIDTH_PIXELS: usize = _usize(option_env!("MINIMAP_WIDTH_PIXELS"), "2048");
pub const MINIMAP_HEIGHT_PIXELS: usize = _usize(option_env!("MINIMAP_HEIGHT_PIXELS"), "2048");

const _: () = assert!(
    WORLD_WIDTH.is_multiple_of(REGION_WIDTH),
    "World width mut be divisible by region width"
);
const _: () = assert!(
    WORLD_HEIGHT.is_multiple_of(REGION_HEIGHT),
    "World height mut be divisible by region height"
);

const fn _usize(s: Option<&str>, s2: &str) -> usize {
    let s = match s {
        Some(s) => s,
        None => s2,
    };
    let bytes = s.as_bytes();
    let mut result = 0usize;
    let mut i = 0;
    while i < bytes.len() {
        result = result * 10 + (bytes[i] - b'0') as usize;
        i += 1;
    }
    result
}

pub trait Client: Clone + std::hash::Hash + Eq + std::fmt::Debug + Send + Sync + 'static {}
impl<T: Clone + std::hash::Hash + Eq + std::fmt::Debug + Send + Sync + 'static> Client for T {}
