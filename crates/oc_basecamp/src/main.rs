use eframe::egui;

mod about;
mod app;
mod battle;
mod errors;
mod map;
mod mods;
mod page;
mod pages;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([900.0, 600.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Ooen Combat",
        options,
        Box::new(|_cc| Ok(Box::new(app::App::new()))),
    )
}
