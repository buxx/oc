use bevy::prelude::*;

#[derive(Debug, Event)]
pub struct ToggleBattleDebugWindow;

#[derive(Debug, Default)]
pub struct Window;

pub fn on_toggle_battle_debug_window(
    _: On<ToggleBattleDebugWindow>,
    mut window: ResMut<crate::states::Window>,
) {
    if let Some(crate::window::Window::BattleDebug(_)) = window.0 {
        tracing::debug!("Close battle debug window");
        window.0 = None;
    } else {
        tracing::debug!("Open battle debug window");
        window.0 = Some(crate::window::Window::BattleDebug(Window));
    }
}
