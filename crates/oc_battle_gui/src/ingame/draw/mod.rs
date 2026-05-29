pub mod individual;
pub mod world;

pub const Z_REGION_BACKGROUND: f32 = 0.0;
#[cfg(feature = "debug")]
pub const Z_TERRAIN_TILE: f32 = 0.1;
pub const Z_INDIVIDUAL: f32 = 2.0;
pub const Z_PROJECTILE: f32 = 3.0;
#[cfg(feature = "debug")]
pub const Z_SELECT_WIRES: f32 = 5.0;
#[cfg(feature = "debug")]
pub const Z_REGION_WIREFRAME: f32 = 1.0;
pub const Z_LOV: f32 = 10.0;
#[cfg(feature = "debug")]
pub const Z_WORLD_CURSOR: f32 = 1.0;
pub const Z_WORLD_MAP_BACKGROUND: f32 = 0.0;
pub const Z_WORLD_MAP_MINIMAP: f32 = 0.1;
pub const Z_WORLD_MAP_BATTLE_SQUARE: f32 = 2.0;
