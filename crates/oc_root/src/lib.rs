pub const WORLD_WIDTH: usize = 10_000;
pub const WORLD_HEIGHT: usize = 10_000;
pub const REGION_WIDTH: usize = 1_000;
pub const REGION_HEIGHT: usize = 1_000;

pub const TILES_COUNT: usize = WORLD_WIDTH * WORLD_HEIGHT;
pub const REGIONS_COUNT: usize = TILES_COUNT / (REGION_WIDTH * REGION_HEIGHT);
pub const REGIONS_WIDTH: usize = WORLD_WIDTH / REGION_WIDTH;
pub const REGIONS_HEIGHT: usize = WORLD_HEIGHT / REGION_HEIGHT;

pub const INDIVIDUALS_COUNT: usize = 1;
pub const INDIVIDUAL_TICK_INTERVAL_US: u64 = 1_000_000 / 1;

const _: () = assert!(
    WORLD_WIDTH % REGION_WIDTH == 0,
    "World width mut be divisible by region width"
);
const _: () = assert!(
    WORLD_HEIGHT % REGION_HEIGHT == 0,
    "World height mut be divisible by region height"
);
