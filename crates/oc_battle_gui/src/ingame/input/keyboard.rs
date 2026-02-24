use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;

use crate::ingame::camera;
use crate::ingame::input::map::{SwitchToBattleMap, SwitchToWorldMap};

pub fn on_key_press(
    mut commands: Commands,
    mut keyboard: MessageReader<KeyboardInput>,
    camera: Res<camera::State>,
) {
    for event in keyboard.read() {
        match event.key_code {
            KeyCode::F1 => match camera.focus {
                camera::Focus::Battle => commands.trigger(SwitchToWorldMap),
                camera::Focus::World => commands.trigger(SwitchToBattleMap),
            },
            _ => {}
        }
    }
}
