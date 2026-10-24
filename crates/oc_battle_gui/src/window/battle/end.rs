use bevy_egui::EguiContexts;

#[derive(Clone, Default)]
pub struct Window;

impl Window {
    pub fn show(&mut self, contexts: &mut EguiContexts) -> bevy::prelude::Result {
        let ctx = contexts.ctx_mut()?;
        bevy_egui::egui::Window::new("BattleEnd")
            .pivot(bevy_egui::egui::Align2::CENTER_CENTER)
            .default_pos(ctx.content_rect().center())
            .show(ctx, |ui| {
                ui.label("Battle is end");
            });

        Ok(())
    }
}
