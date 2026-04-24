use bevy::input::ButtonState;
use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;

use crate::ingame::camera;
#[cfg(feature = "debug")]
use crate::ingame::camera::debug::tile::ToggleShowTiles;
use crate::ingame::camera::map::SaveCurrentWindowCenterAsBattleCenter;
use crate::ingame::input::map::{SwitchToBattleMap, SwitchToWorldMap};
use crate::window::ToggleWindow;
use crate::window::Window;
#[cfg(feature = "debug")]
use crate::window::debug::battle::DebugBattleWindow;
#[cfg(feature = "debug")]
use crate::window::debug::battle::window::Window as DebugWindow;
use crate::window::menu::battle::{BattleMenuWindow, Window as BattleMenu};

pub fn on_key_press(
    mut commands: Commands,
    mut keyboard: MessageReader<KeyboardInput>,
    camera: Res<camera::State>,
    menu: Res<BattleMenuWindow>,
    #[cfg(feature = "debug")] debug: Res<DebugBattleWindow>,
) {
    for event in keyboard.read() {
        match (event.state, event.key_code) {
            (ButtonState::Released, KeyCode::F1) => match camera.focus {
                camera::Focus::Battle => {
                    commands.trigger(SaveCurrentWindowCenterAsBattleCenter);
                    commands.trigger(SwitchToWorldMap);
                }
                camera::Focus::World => commands.trigger(SwitchToBattleMap),
            },
            (ButtonState::Released, KeyCode::Escape) => {
                let window = menu.0.clone().unwrap_or(BattleMenu);
                commands.trigger(ToggleWindow(Window::BattleMenu(window)));
            }
            #[cfg(feature = "debug")]
            (ButtonState::Released, KeyCode::F11) => {
                commands.trigger(ToggleShowTiles);
            }
            #[cfg(feature = "debug")]
            (ButtonState::Released, KeyCode::F12) => {
                let window = debug.0.clone().unwrap_or(DebugWindow::default());
                commands.trigger(ToggleWindow(Window::BattleDebug(window)));
            }
            _ => {}
        }
    }
}
