use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use crate::states;

#[cfg(feature = "debug")]
pub mod debug;

#[derive(Debug)]
pub enum Window {
    #[cfg(feature = "debug")]
    BattleDebug(debug::battle::Window),
}
impl Window {
    fn show(&self, contexts: &mut EguiContexts) -> Result {
        egui::Window::new("Hello").show(contexts.ctx_mut()?, |ui| {
            ui.label("world");
        });

        Ok(())
    }
}

pub struct WindowPlugin;

impl Plugin for WindowPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(EguiPrimaryContextPass, show);

        #[cfg(feature = "debug")]
        {
            app.add_observer(debug::battle::on_toggle_battle_debug_window);
        }
    }
}

fn show(mut contexts: EguiContexts, mut window: ResMut<states::Window>) -> Result {
    if let Some(window) = &mut window.0 {
        window.show(&mut contexts)?
    }

    Ok(())
}
