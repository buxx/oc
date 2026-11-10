use crate::pages::PageContext;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Home,
    Solo,
    Multiplayer,
    Campaign,
    Battle,
}

impl Page {
    pub fn title(&self) -> &'static str {
        match self {
            Page::Home => "Home",
            Page::Solo => "Solo",
            Page::Multiplayer => "Multiplayer",
            Page::Campaign => "Campaign",
            Page::Battle => "Battle",
        }
    }

    pub fn path(&self) -> Vec<Page> {
        match self {
            Page::Home => vec![Page::Home],
            Page::Solo | Page::Multiplayer => vec![Page::Home, *self],
            Page::Campaign | Page::Battle => vec![Page::Home, Page::Solo, *self],
        }
    }

    pub fn show(&self, ui: &mut eframe::egui::Ui, ctx: &mut PageContext) -> Option<Page> {
        use crate::pages;
        match self {
            Page::Home => pages::home::show(ui),
            Page::Solo => pages::solo::show(ui),
            Page::Multiplayer => pages::multiplayer::show(ui),
            Page::Campaign => pages::campaign::show(ui),
            Page::Battle => pages::battle::show(ui, ctx),
        }
    }
}
