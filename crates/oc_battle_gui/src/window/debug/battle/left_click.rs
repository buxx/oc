use bevy::prelude::*;
use bevy_egui::prelude::*;
use oc_mod::{Mod, weapons::WeaponType};
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
            LeftClickModeType::Select => {
                commands.trigger(SetLeftClick(LeftClickMode::Select));
            }
            LeftClickModeType::SpawnProjectile => {
                let weapon_type_before = self.spawn_weapon_type.clone();
                let weapon_type = &mut self.spawn_weapon_type;
                let weapon_before = self.spawn_weapon.clone();
                let weapon = &mut self.spawn_weapon;
                let ammunition = &mut self.spawn_ammunition;
                let shot = &mut self.spawn_shot;
                let repeat = &mut self.spawn_repeat;
                let click_mode = &mut self.spawn_projectile_click_mode;
                let plus_z = &mut self.spawn_projectile_plus_z;

                ui.horizontal(|ui| {
                    egui::ComboBox::new("weapon_type", "")
                        .selected_text(weapon_type.name())
                        .show_ui(ui, |ui| {
                            for item in WeaponType::iter() {
                                let name = item.name();
                                ui.selectable_value(weapon_type, item, name);
                            }
                        });

                    egui::ComboBox::new("weapon", "")
                        .selected_text(weapon.as_ref().map(|p| p.name()).unwrap_or_default())
                        .show_ui(ui, |ui| {
                            let items = mod_.weapons.iter().filter(|p| p.is_type(*weapon_type));
                            for item in items {
                                let item = item.clone();
                                let name = item.name();
                                ui.selectable_value(weapon, Some(item.clone()), name);
                            }
                        });

                    if let Some(weapon) = &weapon {
                        egui::ComboBox::new("ammunition", "")
                            .selected_text(
                                ammunition.as_ref().map(|p| p.name()).unwrap_or_default(),
                            )
                            .show_ui(ui, |ui| {
                                for item in weapon.ammunitions() {
                                    let item = item.clone();
                                    let name = item.name();
                                    ui.selectable_value(ammunition, Some(item.clone()), name);
                                }
                            });

                        egui::ComboBox::new("shot_mode", "")
                            .selected_text(shot.as_ref().map(|p| p.name()).unwrap_or_default())
                            .show_ui(ui, |ui| {
                                for item in weapon.shots() {
                                    let item = item.clone();
                                    let name = item.name();
                                    ui.selectable_value(shot, Some(item.clone()), name);
                                }
                            });
                    }

                    egui::ComboBox::new("click_mode", "")
                        .selected_text(click_mode.to_string())
                        .show_ui(ui, |ui| {
                            for item in super::SpawnProjectileClickMode::iter() {
                                let name = item.to_string();
                                ui.selectable_value(click_mode, item, name);
                            }
                        });
                });

                ui.horizontal(|ui| {
                    ui.add(egui::DragValue::new(repeat).range(1..=255).speed(1));
                    ui.label("count");

                    ui.separator();

                    ui.add(
                        egui::DragValue::new(&mut plus_z.0)
                            .range((0.0)..=(5.0))
                            .speed(0.1),
                    );
                    ui.label("+z");
                });

                if weapon_type_before != self.spawn_weapon_type {
                    self.spawn_weapon = None;
                    self.spawn_ammunition = None;
                    self.spawn_shot = None;
                }

                if weapon_before != self.spawn_weapon {
                    self.spawn_ammunition = None;
                    self.spawn_shot = None;
                }

                if let (Some(weapon), Some(ammunition), Some(shot)) =
                    (&self.spawn_weapon, &self.spawn_ammunition, &self.spawn_shot)
                {
                    let spawn = SpawnProjectileProfile::new(
                        weapon.index(),
                        ammunition.index(),
                        shot.index(),
                        *repeat,
                        *plus_z,
                    );
                    commands.trigger(SetLeftClick(LeftClickMode::SpawnProjectile(spawn)));
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
