use std::time::Duration;

use eframe::egui;

use crate::page::Page;

use super::PageContext;

/// Delay between two checks of battle gui process
const POLL_INTERVAL: Duration = Duration::from_secs(1);

/// Wait battle gui process exits
pub fn show(ui: &mut egui::Ui, ctx: &mut PageContext) -> Option<Page> {
    let Some(process) = ctx.battle_process.as_mut() else {
        ui.label("No battle in progress");
        return Some(Page::ChooseSquads);
    };

    match process.try_wait() {
        Ok(None) => {
            ui.centered_and_justified(|ui| {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label("Battle in progress ...");
                });
            });
            ui.ctx().request_repaint_after(POLL_INTERVAL);
            None
        }
        Ok(Some(status)) => {
            *ctx.battle_process = None;
            if status.success() {
                Some(Page::Battle)
            } else {
                ctx.errors
                    .push(format!("Battle exited with error ({status})"));
                *ctx.show_errors = true;
                Some(Page::ChooseSquads)
            }
        }
        Err(error) => {
            *ctx.battle_process = None;
            ctx.errors.push(format!("Wait battle: {error}"));
            *ctx.show_errors = true;
            Some(Page::ChooseSquads)
        }
    }
}
