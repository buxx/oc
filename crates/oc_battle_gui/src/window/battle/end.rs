use bevy::{app::AppExit, prelude::Commands};
use bevy_egui::EguiContexts;

#[derive(Clone, Default)]
pub struct Window;

impl Window {
    pub fn show(
        &mut self,
        contexts: &mut EguiContexts,
        commands: &mut Commands,
    ) -> bevy::prelude::Result {
        let ctx = contexts.ctx_mut()?;
        bevy_egui::egui::Window::new("BattleEnd")
            .pivot(bevy_egui::egui::Align2::CENTER_CENTER)
            .default_pos(ctx.content_rect().center())
            .show(ctx, |ui| {
                ui.label("Battle is end");
                if ui.button("Close").clicked() {
                    commands.write_message(AppExit::Success);
                }
            });

        Ok(())
    }
}
