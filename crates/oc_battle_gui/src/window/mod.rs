use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass};
use oc_mod::Mod;
use oc_root::WorldConfig;
use oc_utils::let_some;

use crate::{
    states::{self, PointerIn},
    window::menu::battle::BattleMenuWindowPlugin,
};

#[cfg(feature = "debug")]
pub mod debug;
pub mod menu;

// TODO: There is a lot of common code for windows, use generic
#[derive(Clone)]
pub enum Window {
    BattleMenu(menu::battle::Window),
    #[cfg(feature = "debug")]
    BattleDebug(debug::battle::window::Window),
}

impl Window {
    fn show(
        &mut self,
        contexts: &mut EguiContexts,
        commands: &mut Commands,
        mod_: &Mod,
        w: &WorldConfig,
    ) -> Result {
        match self {
            Window::BattleMenu(window) => window.show(contexts, commands, mod_, w)?,
            #[cfg(feature = "debug")]
            Window::BattleDebug(window) => window.show(contexts, commands, mod_, w)?,
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
    mut pointer: ResMut<NextState<PointerIn>>,
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

    *pointer = NextState::Pending(PointerIn::Battle)
}

pub struct WindowPlugin;

impl Plugin for WindowPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(BattleMenuWindowPlugin)
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
    g: Res<states::GameConfig>,
    mut pointer: ResMut<NextState<PointerIn>>,
) -> Result {
    let_some!(window = &mut window.0, return Ok(()));
    let_some!(g = &g.0, return Ok(()));

    window.show(&mut contexts, &mut commands, &g.mod_, &g.w)?;

    match contexts.ctx_mut()?.is_pointer_over_egui() {
        true => *pointer = NextState::Pending(PointerIn::Window),
        false => *pointer = NextState::Pending(PointerIn::Battle),
    }

    Ok(())
}
