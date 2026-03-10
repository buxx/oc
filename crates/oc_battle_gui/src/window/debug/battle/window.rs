use std::time::Instant;

use bevy::prelude::*;
use bevy_egui::EguiContexts;
use egui_dock::{DockArea, DockState, Style};
use strum::IntoEnumIterator;

use crate::ingame::camera::debug::tile::ToggleShowTiles;

#[derive(Clone)]
pub struct Window {
    pub tree: DockState<super::Tab>,
    pub last: Instant,
    pub context: super::Context,
}
impl Default for Window {
    fn default() -> Self {
        Self {
            tree: DockState::new(super::Tab::iter().collect()),
            last: Instant::now(),
            context: Default::default(),
        }
    }
}

impl Window {
    pub fn show(&mut self, contexts: &mut EguiContexts, commands: &mut Commands) -> Result {
        let ctx = contexts.ctx_mut()?;
        let size = bevy_egui::egui::vec2(600.0, 400.0);
        let mut context = super::InContext::new(&mut self.context, commands);

        bevy_egui::egui::Window::new("Dock window")
            .default_size(size)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let show_tiles = &mut context.context.show_tiles;
                    if ui.checkbox(show_tiles, "Tiles").changed() {
                        context.commands.trigger(ToggleShowTiles);
                    }
                });

                DockArea::new(&mut self.tree)
                    .style(Style::from_egui(ctx.style().as_ref()))
                    .show_inside(ui, &mut context);
            });

        Ok(())
    }
}
