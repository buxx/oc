use bevy::prelude::*;
use bevy_egui::egui;
use oc_mod::Mod;

impl super::Context {
    pub fn ui_cursor(&mut self, ui: &mut egui::Ui, _commands: &mut Commands, _mod_: &Mod) {
        ui.label(format!("cursor: {:?}", self.cursor));
        ui.label(format!("point: {:?}", self.point));
        ui.label(format!("tile: {:?}", self.tile));
        ui.label(format!("region: {:?}", self.region));
    }
}
