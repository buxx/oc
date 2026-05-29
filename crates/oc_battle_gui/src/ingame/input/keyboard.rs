use bevy::input::ButtonState;
use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;

#[cfg(feature = "debug")]
use crate::ingame::camera::debug::tile::ToggleShowTiles;
use crate::ingame::camera::map::SaveCurrentWindowCenterAsBattleCenter;
use crate::ingame::{QuitHeightMap, RestoreBattleCenter, SwitchToBattleMap, SwitchToWorldMap};
use crate::ingame::{SwitchToHeightMap, camera};
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
                    tracing::debug!("Trigger switch to world map (from battle map)");
                    commands.trigger(SaveCurrentWindowCenterAsBattleCenter);
                    commands.trigger(SwitchToWorldMap);
                }
                camera::Focus::Height => {}
                camera::Focus::World => {
                    tracing::debug!("Trigger switch to battle map (from world map)");
                    commands.trigger(RestoreBattleCenter);
                    commands.trigger(SwitchToBattleMap); // Todo refact and trigger unmount world
                }
            },
            (ButtonState::Released, KeyCode::F2) => match camera.focus {
                camera::Focus::Battle => {
                    tracing::debug!("Trigger switch to height map (from battle map)");
                    commands.trigger(SaveCurrentWindowCenterAsBattleCenter);
                    commands.trigger(SwitchToHeightMap);
                }
                camera::Focus::Height => {
                    tracing::debug!("Trigger switch to battle map (from height map)");
                    commands.trigger(QuitHeightMap);
                    commands.trigger(RestoreBattleCenter);
                    commands.trigger(SwitchToBattleMap); // Todo refact and trigger unmount height
                }
                camera::Focus::World => {}
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
