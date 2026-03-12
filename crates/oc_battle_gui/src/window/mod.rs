use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass};
use oc_mod::Mod;

use crate::states;

#[cfg(feature = "debug")]
pub mod debug;

#[derive(Deref, DerefMut, Resource, Default)]
pub struct PointerInWindow(pub bool);

#[derive(Clone)]
pub enum Window {
    #[cfg(feature = "debug")]
    BattleDebug(debug::battle::window::Window),
}

impl Window {
    fn show(&mut self, contexts: &mut EguiContexts, commands: &mut Commands, mod_: &Mod) -> Result {
        #[cfg(feature = "debug")]
        match self {
            Window::BattleDebug(window) => window.show(contexts, commands, mod_)?,
        }

        Ok(())
    }
}

#[derive(Event)]
pub struct ToggleWindow(pub Window);

#[derive(Event)]
pub struct MountedWindow(pub Window);

#[derive(Event)]
pub struct UnmountedWindow(pub Window);

pub fn on_toggle_debug_window(
    toggle: On<ToggleWindow>,
    mut commands: Commands,
    mut window: ResMut<crate::states::Window>,
) {
    if let Some(window_) = &window.0 {
        if std::mem::discriminant(window_) == std::mem::discriminant(&toggle.0) {
            commands.trigger(UnmountedWindow(window_.clone()));
            window.0 = None;
        }
    } else {
        window.0 = Some(toggle.0.clone());
        commands.trigger(MountedWindow(toggle.0.clone()));
    }
}

pub struct WindowPlugin;

impl Plugin for WindowPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PointerInWindow>()
            .add_systems(EguiPrimaryContextPass, show)
            .add_observer(on_toggle_debug_window);

        #[cfg(feature = "debug")]
        {
            app.add_plugins(debug::battle::DebugBattleWindowPlugin::default());
        }
    }
}

fn show(
    mut contexts: EguiContexts,
    mut window: ResMut<states::Window>,
    mut commands: Commands,
    mod_: Res<states::Mod>,
    mut pointer: ResMut<PointerInWindow>,
) -> Result {
    let Some(window) = &mut window.0 else {
        return Ok(());
    };
    let Some(mod_) = &mod_.0 else {
        return Ok(());
    };

    window.show(&mut contexts, &mut commands, &mod_)?;
    pointer.0 = contexts.ctx_mut()?.is_pointer_over_area();

    Ok(())
}
