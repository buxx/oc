use bevy::prelude::*;
use oc_physics::fx::{Audio, Fx};
use oc_root::files;

use crate::{
    network,
    states::{Meta, Mod, StaticSource},
};

#[derive(Debug, Event)]
pub struct FxEvent(pub Fx);

#[derive(Component, Default)]
struct Emitter;

pub struct FxPlugin;

// See https://taintedcoders.com/bevy/audio
impl Plugin for FxPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_fx)
            .add_systems(Startup, setup)
            .add_systems(Update, update_listener);
    }
}

fn setup(mut commands: Commands) {
    // Space between the two ears
    // FIXME: window width / 2
    let gap = 800.0; // FIXME: update when window size change

    commands.spawn((Transform::default(), SpatialListener::new(gap)));
}

fn on_fx(
    fx: On<FxEvent>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mod_: Res<Mod>,
    meta: Res<Meta>,
    sources: Res<StaticSource>,
    network: Res<network::state::State>,
) {
    let Some(mod_) = &mod_.0 else { return };
    let Some(static_) = &sources.0 else { return };
    let Some(meta) = &meta.0 else { return };
    let Some(connect) = network.server.clone() else {
        return;
    };
    let mod__ = mod_.name().to_string();
    let world = meta.canonical();
    let files = files::Files::new(mod__, world).into_gui(static_.clone(), connect.into());
    let path = files.mod_().join("sounds");

    match &fx.0 {
        Fx::Audio(audio) => match audio {
            Audio::PlayOnce(sound, position) => {
                let sound = mod_.sound(*sound);
                let path = path.join(&sound.name);
                commands.spawn((
                    Emitter,
                    Transform::from_translation(Vec3::new(position[0], position[1], 0.0)),
                    AudioPlayer::new(asset_server.load(path)),
                    PlaybackSettings::ONCE.with_spatial(true),
                ));
            }
        },
    }
}

fn update_listener(
    camera: Single<&Transform, (With<Camera2d>, Without<SpatialListener>)>,
    mut listener: Single<&mut Transform, With<SpatialListener>>,
) {
    listener.translation = camera.translation;
}
