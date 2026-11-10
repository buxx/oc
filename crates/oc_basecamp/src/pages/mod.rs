use eframe::egui;

use crate::{battle::Battle, map::Map, page::Page};

pub struct PageContext<'a> {
    pub battles: &'a [Battle],
    pub maps: &'a [Map],
    pub selected_battle: &'a mut Option<usize>,
    pub show_all_battles: &'a mut bool,
    pub selected_mod: Option<&'a str>,
}

pub mod battle;
pub mod campaign;
pub mod home;
pub mod multiplayer;
pub mod solo;

pub fn button_row(ui: &mut egui::Ui, items: [(&str, Page); 2]) -> Option<Page> {
    let mut clicked = None;
    let spacing = ui.spacing().item_spacing;
    let avail = ui.available_size();
    let size = egui::vec2(((avail.x - spacing.x) / 2.0).max(50.0), avail.y.min(200.0));

    ui.horizontal(|ui| {
        for (label, page) in items {
            let button = egui::Button::new(egui::RichText::new(label).size(28.0));
            if ui.add_sized(size, button).clicked() {
                clicked = Some(page);
            }
        }
    });

    clicked
}

pub fn not_implemented(ui: &mut egui::Ui) -> Option<Page> {
    ui.label("not yet implemented");
    None
}
