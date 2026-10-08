use bevy::prelude::*;

use crate::ingame::hud::HUD_HEIGHT;

pub const MINIMAP_SIZE: f32 = HUD_HEIGHT;

#[derive(Component, Default, Clone)]
pub struct Minimap;

pub fn show() -> impl Scene {
    bsn! {
        Minimap
        Node {
            width: px(MINIMAP_SIZE),
            height: px(MINIMAP_SIZE),
            flex_shrink: 0.,
        }
        BackgroundColor(Color::BLACK)
    }
}
