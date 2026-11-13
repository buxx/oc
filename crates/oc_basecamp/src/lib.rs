//! Basecamp: the launcher application (mods, battles, deployments).
//! Exposed as a library so end-to-end tests can drive [`app::App`].

mod about;
pub mod app;
mod battle;
mod errors;
mod launch;
mod map;
mod map_view;
mod minimap;
mod mods;
pub mod page;
mod pages;

pub const UI_ZOOM: f32 = 1.25;
