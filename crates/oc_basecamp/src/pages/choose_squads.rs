use eframe::egui;

use crate::{launch, map_view, minimap, page::Page};

use super::PageContext;

pub fn show(ui: &mut egui::Ui, ctx: &mut PageContext) -> Option<Page> {
    let Some(deployment) = ctx.deployment.as_ref() else {
        ui.label("No deployment loaded");
        return None;
    };

    let Some((battle, scenario)) = ctx.selected() else {
        ui.label("No scenario selected");
        return None;
    };
    let map_path = minimap::scenario_path(ctx.app_dir, battle, scenario, ctx.selected_mod);

    let squads: Vec<&str> = deployment
        .squads
        .iter()
        .map(|squad| squad.label.as_str())
        .collect();

    egui::Panel::left("squads_list")
        .resizable(true)
        .default_size(220.0)
        .show(ui, |ui| {
            map_view::button(ui, "choose_squads", &map_path);

            ui.separator();

            ui.strong("Squads");

            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                for (index, label) in squads.iter().enumerate() {
                    let placed = ctx.squad_slots.contains(&Some(index));
                    let response = ui.add_enabled(!placed, egui::Button::selectable(false, *label));
                    if response.clicked()
                        && let Some(slot) = ctx.squad_slots.iter_mut().find(|s| s.is_none())
                    {
                        *slot = Some(index);
                    }
                }
            });
        });

    let mut start = false;
    egui::Panel::bottom("choose_squads_actions").show(ui, |ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let ready = ctx.squad_slots.iter().any(Option::is_some);
            let button = egui::Button::new(egui::RichText::new("Start battle").size(20.0));
            let response = ui
                .add_enabled(ready, button)
                .on_disabled_hover_text("Choose at least one squad");
            start = response.clicked();
        });
    });

    egui::CentralPanel::default().show(ui, |ui| {
        ui.strong("Slots");
        ui.separator();

        for (number, slot) in ctx.squad_slots.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                ui.label(format!("{}.", number + 1));
                match slot.and_then(|index| squads.get(index)) {
                    Some(label) => {
                        ui.label(*label);
                        if ui.button("Remove").clicked() {
                            *slot = None;
                        }
                    }
                    None => {
                        ui.weak("empty");
                    }
                }
            });
        }
    });

    if !start {
        return None;
    }

    match launch::start(ctx) {
        Ok(child) => {
            *ctx.battle_process = Some(child);
            Some(Page::BattleInProgress)
        }
        Err(error) => {
            ctx.errors.push(format!("Start battle: {error}"));
            *ctx.show_errors = true;
            None
        }
    }
}
