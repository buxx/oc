use bevy::prelude::*;
use bevy_egui::prelude::*;
use oc_mod::{Mod, projectiles::ProjectileType};
use strum::IntoEnumIterator;

use crate::ingame::{
    debug::projectile::SpawnProjectileProfile,
    input::left_click::{LeftClickMode, LeftClickModeType, SetLeftClick},
};

impl super::Context {
    // TODO: seems we can improve this code
    pub fn ui_left_click(&mut self, ui: &mut egui::Ui, commands: &mut Commands, mod_: &Mod) {
        let left_click_mode_before = self.left_click_mode.clone();
        let left_click_mode = &mut self.left_click_mode;

        egui::ComboBox::from_label("Left click mode")
            .selected_text(left_click_mode.name())
            .show_ui(ui, |ui| {
                for item in LeftClickModeType::iter() {
                    let item_ = item.clone();
                    let name = item_.name();
                    ui.selectable_value(left_click_mode, item, name);
                }
            });

        match left_click_mode {
            LeftClickModeType::Select => {}
            LeftClickModeType::SpawnProjectile => {
                let projectile_type = &mut self.spawn_projectile_type;
                let projectile_before = self.spawn_projectile.clone();
                let projectile = &mut self.spawn_projectile;
                let profile = &mut self.spawn_profile;

                egui::ComboBox::from_label("Projectile type")
                    .selected_text(projectile_type.name())
                    .show_ui(ui, |ui| {
                        for item in ProjectileType::iter() {
                            let name = item.name();
                            ui.selectable_value(projectile_type, item, name);
                        }
                    });

                egui::ComboBox::from_label("Projectile")
                    .selected_text(projectile.as_ref().map(|p| p.label()).unwrap_or_default())
                    .show_ui(ui, |ui| {
                        let items = mod_
                            .projectiles
                            .iter()
                            .filter(|p| p.is_type(*projectile_type));
                        for item in items {
                            let item = item.clone();
                            let name = item.label();
                            ui.selectable_value(projectile, Some(item.clone()), name);
                        }
                    });

                ui.horizontal(|ui| {
                    ui.add(
                        egui::DragValue::new(&mut profile.count)
                            .range(1..=32)
                            .speed(1),
                    );
                    ui.label("count");

                    ui.separator();

                    ui.add(
                        egui::DragValue::new(&mut profile.interval_ms)
                            .range(10..=5000)
                            .speed(10),
                    );
                    ui.label("interval");
                });

                if projectile_before != self.spawn_projectile {
                    if let Some(spawn_projectile) = &self.spawn_projectile {
                        let projectile = spawn_projectile.clone();
                        let profile = profile.clone();
                        let profile = SpawnProjectileProfile::new(projectile, profile);
                        commands.trigger(SetLeftClick(LeftClickMode::SpawnProjectile(profile)));
                    }
                }
            }
        }

        if left_click_mode_before != self.left_click_mode {
            match self.left_click_mode {
                LeftClickModeType::Select => {
                    commands.trigger(SetLeftClick(LeftClickMode::Select));
                }
                _ => {}
            }
        }
    }
}
