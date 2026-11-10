use eframe::egui;

use crate::{
    about,
    battle::{self, Battle},
    errors,
    map::{self, Map},
    mods,
    page::Page,
    pages::PageContext,
};

pub struct App {
    loading: bool,
    page: Page,
    mods: Vec<String>,
    selected_mod: Option<String>,
    show_about: bool,
    battles: Vec<Battle>,
    maps: Vec<Map>,
    selected_battle: Option<usize>,
    selected_scenario: Option<usize>,
    show_all_battles: bool,
    errors: Vec<String>,
    show_errors: bool,
}

impl App {
    pub fn new() -> Self {
        let mods = mods::list();
        let selected_mod = mods.first().cloned();

        Self {
            page: Page::Home,
            mods,
            selected_mod,
            show_about: false,
            battles: vec![],
            maps: vec![],
            selected_battle: None,
            selected_scenario: None,
            show_all_battles: false,
            loading: true,
            errors: vec![],
            show_errors: false,
        }
    }

    fn menu_bar(&mut self, ui: &mut egui::Ui) {
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Quit").clicked() {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });
            ui.menu_button("Help", |ui| {
                if ui.button("About").clicked() {
                    self.show_about = true;
                    ui.close();
                }
            });
        });
    }

    fn navigation(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            self.errors(ui);
            self.breadcrumbs(ui);
            self.mods(ui);
        });
    }

    fn mods(&mut self, ui: &mut egui::Ui) {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let selected_text = self.selected_mod.clone().unwrap_or_else(|| "(none)".into());

            // Mod can only be changed from the Home page
            ui.add_enabled_ui(self.page == Page::Home, |ui| {
                egui::ComboBox::from_id_salt("mod_select")
                    .selected_text(selected_text)
                    .show_ui(ui, |ui| {
                        for name in &self.mods {
                            ui.selectable_value(&mut self.selected_mod, Some(name.clone()), name);
                        }
                    });
            });

            ui.label("Mod");
        });
    }

    fn breadcrumbs(&mut self, ui: &mut egui::Ui) {
        let path = self.page.path();
        let last = path.len() - 1;
        for (i, page) in path.iter().enumerate() {
            if i == last {
                ui.strong(page.title());
            } else {
                if ui.link(page.title()).clicked() {
                    self.page = *page;
                }
                ui.label(">");
            }
        }
    }

    fn errors(&mut self, ui: &mut egui::Ui) {
        if !self.errors.is_empty() {
            let icon = egui::RichText::new("⚠").color(ui.visuals().error_fg_color);
            if ui.button(icon).on_hover_text("Errors").clicked() {
                self.show_errors = true;
            }
        }
    }

    fn loading(&mut self, ui: &mut egui::Ui) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.centered_and_justified(|ui| {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label("Loading...");
                });
            });
        });

        let (battles, errors) = battle::load_all();
        self.battles = battles;
        self.errors.extend(errors);

        let (maps, errors) = map::load_all();
        self.maps = maps;
        self.errors.extend(errors);

        self.loading = false;
        ui.ctx().request_repaint();
    }

    fn show(&mut self, ui: &mut egui::Ui) {
        egui::Panel::top("navigation").show(ui, |ui| self.navigation(ui));
        egui::CentralPanel::default().show(ui, |ui| {
            let mut ctx = PageContext {
                battles: &self.battles,
                maps: &self.maps,
                selected_battle: &mut self.selected_battle,
                selected_scenario: &mut self.selected_scenario,
                show_all_battles: &mut self.show_all_battles,
                selected_mod: self.selected_mod.as_deref(),
            };
            if let Some(next) = self.page.show(ui, &mut ctx) {
                self.page = next;
            }
        });
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::top("menu_bar").show(ui, |ui| self.menu_bar(ui));

        if self.loading {
            self.loading(ui);
        } else {
            self.show(ui);
        }

        if self.show_errors && errors::show(ui.ctx(), &mut self.show_errors, &self.errors) {
            self.errors.clear();
            self.show_errors = false;
        }

        if self.show_about {
            about::show(ui.ctx(), &mut self.show_about);
        }
    }
}
