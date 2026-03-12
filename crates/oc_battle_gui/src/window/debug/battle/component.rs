use std::fmt::Display;

use bevy::prelude::*;
use bevy_egui::prelude::*;
use oc_geo::region::RegionXy;
use oc_mod::Mod;

use crate::{
    ingame::camera::{GoToPoint, region::Region},
    window::debug::{battle::View, subject::Subject},
};

impl super::Context {
    pub fn ui_components(&mut self, ui: &mut egui::Ui, commands: &mut Commands, mod_: &Mod) {
        ui.horizontal(|ui| {
            if ui.button("x").clicked() {
                self.view = View::None;
            }
            if ui
                .button(format!("Regions ({})", self.regions.len()))
                .clicked()
            {
                self.view = View::Regions;
            };
            if ui
                .button(format!("Individuals ({})", self.individuals.len()))
                .clicked()
            {
                self.view = View::Individuals;
            };
            if ui
                .button(format!("Projectiles ({})", self.projectiles.len()))
                .clicked()
            {
                self.view = View::Projectiles;
            };
        });

        if let Some(action) = match self.view {
            View::None => None,
            View::Regions => self.ui_regions(ui, &self.regions),
            View::Individuals => self.ui_subjects(ui, &self.individuals),
            View::Projectiles => self.ui_subjects(ui, &self.projectiles),
        } {
            match action {
                Action::GoToPoint(point) => {
                    commands.trigger(GoToPoint(point));
                }
            }
        }
    }

    fn ui_regions(&self, ui: &mut egui::Ui, regions: &Vec<Region>) -> Option<Action> {
        ui.vertical(|ui| {
            for region in regions {
                let region: RegionXy = region.0.into();
                ui.label(&format!("{}.{}", region.0.0, region.0.1));
            }
        });
        None
    }

    fn ui_subjects<I: Display>(
        &self,
        ui: &mut egui::Ui,
        subjects: &Vec<Subject<I>>,
    ) -> Option<Action> {
        let mut action = None;

        ui.vertical(|ui| {
            for subject in subjects {
                ui.horizontal(|ui| {
                    let position = subject.physics.position;
                    let region = subject.physics.region;

                    if ui.button("⏵").clicked() {
                        action = Some(Action::GoToPoint(subject.physics.position.clone()));
                    }
                    ui.label(format!(
                        "{} {}.{} ({}.{})",
                        subject.i, position[0], position[1], region.0.0, region.0.1
                    ));
                });
            }
        });

        action
    }
}

pub enum Action {
    GoToPoint([f32; 2]),
}
