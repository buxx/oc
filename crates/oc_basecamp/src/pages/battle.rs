use eframe::egui;

use crate::{battle::Battle, map::Map, page::Page};

use super::PageContext;

pub fn show(ui: &mut egui::Ui, ctx: &mut PageContext) -> Option<Page> {
    egui::Panel::left("battles_list")
        .resizable(true)
        .default_size(220.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.strong("Battle");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.checkbox(ctx.show_all_battles, "Show all").on_hover_text(
                        "Check to list battles with map which is incompatible with selected mod",
                    );
                });
            });

            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                for (i, battle) in ctx.battles.iter().enumerate() {
                    if !is_listed(ctx, battle) {
                        continue;
                    }

                    if ui
                        .selectable_label(*ctx.selected_battle == Some(i), &battle.name)
                        .clicked()
                    {
                        *ctx.selected_battle = Some(i);
                    }
                }
            });
        });

    egui::CentralPanel::default().show(ui, |ui| {
        match ctx
            .selected_battle
            .and_then(|i| ctx.battles.get(i))
            .filter(|b| is_listed(ctx, b))
        {
            Some(battle) => {
                let errors = mod_errors(battle, ctx.maps, ctx.selected_mod);
                ui.horizontal(|ui| {
                    ui.heading(&battle.name);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if let Some(errors) = &errors {
                            compatibility(ui, errors);
                        }
                    });
                });

                ui.separator();

                ui.label(&battle.description);
            }
            None => {
                ui.label("Select a battle");
            }
        }
    });

    None
}

/// Battle is listed if all are shown, or if its map has no error with the selected mod.
fn is_listed(ctx: &PageContext, battle: &Battle) -> bool {
    *ctx.show_all_battles
        || mod_errors(battle, ctx.maps, ctx.selected_mod).is_none_or(|e| e.is_empty())
}

/// Errors of the battle map with the selected mod.
fn mod_errors(battle: &Battle, maps: &[Map], selected_mod: Option<&str>) -> Option<Vec<String>> {
    let selected_mod = selected_mod?;

    let Some(map) = maps.iter().find(|m| m.name == battle.map) else {
        return Some(vec![format!("Map '{}' not found", battle.map)]);
    };
    let Some(compat) = map
        .compatibility
        .iter()
        .find(|c| c.mod_name == selected_mod)
    else {
        return Some(vec![format!("Mod '{selected_mod}' not checked")]);
    };

    Some(compat.errors.clone())
}

/// Error icon button opening a popin listing errors, when errors.
fn compatibility(ui: &mut egui::Ui, errors: &[String]) {
    if errors.is_empty() {
        return;
    }

    let open_id = egui::Id::new("battle_mod_errors_open");
    let mut open = ui.data(|d| d.get_temp::<bool>(open_id)).unwrap_or(false);

    let icon = egui::RichText::new("⚠")
        .color(ui.visuals().error_fg_color)
        .size(20.0);
    if ui
        .button(icon)
        .on_hover_text("Incompatible with selected mod")
        .clicked()
    {
        open = true;
    }

    if open {
        egui::Window::new("Compatibility errors")
            .open(&mut open)
            .collapsible(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ui.ctx(), |ui| {
                egui::ScrollArea::vertical()
                    .max_height(300.0)
                    .show(ui, |ui| {
                        for error in errors {
                            ui.colored_label(ui.visuals().error_fg_color, error);
                        }
                    });
            });
    }

    ui.data_mut(|d| d.insert_temp(open_id, open));
}
