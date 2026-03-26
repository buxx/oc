use std::path::PathBuf;

use bevy::prelude::*;
use oc_physics::fx::{Audio, Fx};

use crate::states::Mod;

#[derive(Debug, Event)]
pub struct FxEvent(pub Fx);

pub struct FxPlugin;

// See https://taintedcoders.com/bevy/audio
impl Plugin for FxPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_fx);
    }
}

fn on_fx(fx: On<FxEvent>, asset_server: Res<AssetServer>, mut commands: Commands, mod_: Res<Mod>) {
    let Some(mod_) = &mod_.0 else { return };

    match &fx.0 {
        Fx::Audio(audio) => {
            // TODO
            let path = PathBuf::from(".cache")
                .join("mods")
                .join(mod_.canonical())
                .join(mod_.name());

            let path = path.join("sounds"); // TODO: const
            match audio {
                // FIXME: spacial effect on volume
                Audio::PlayOnce(sound) => {
                    let sound = mod_.sound(*sound);
                    let path = path.join(&sound.name);
                    commands.spawn((
                        AudioPlayer::new(asset_server.load(path)),
                        PlaybackSettings::ONCE,
                    ));
                }
            }
        }
    }
}
