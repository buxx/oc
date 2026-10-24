use crate::states::GuiFiles;
use bevy::prelude::*;
use oc_physics::fx::{Audio, Fx};
use oc_utils::let_some;

use crate::{ingame::BattlePhase, states::GameConfig};

#[derive(Debug, Event)]
pub struct FxEvent(pub Fx);

#[derive(Component, Default)]
struct Emitter;

/// Too many at once starve the output buffer. Extra spatial sounds are dropped.
const MAX_CONCURRENT_EMITTERS: usize = 24;

pub struct FxPlugin;

// See https://taintedcoders.com/bevy/audio
impl Plugin for FxPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_fx)
            .add_systems(Startup, setup)
            .add_systems(Update, update_listener);
    }
}

pub fn play_phase_start(commands: &mut Commands, g: &GameConfig, phase: BattlePhase) {
    let_some!(g = &g.0, return);
    let groups = &g.mod_.sound_groups;
    let sound = match phase {
        BattlePhase::Deployment => groups.deployment_start,
        BattlePhase::Fight => groups.fight_start,
        BattlePhase::End => None,
    };
    let_some!(sound = sound, return);
    commands.trigger(FxEvent(Fx::Audio(Audio::Play(sound))));
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
    g: Res<GameConfig>,
    files: Res<GuiFiles>,
    emitters: Query<(), With<Emitter>>,
) {
    let_some!(g = &g.0, return);
    let_some!(files = &files.0, return);
    let path = files.mod_().join("sounds");

    match &fx.0 {
        Fx::Audio(audio) => match audio {
            Audio::PlayWithGeo(sound, position) => {
                if emitters.iter().count() >= MAX_CONCURRENT_EMITTERS {
                    return;
                }
                let sound = g.mod_.sound(*sound);
                let path = path.join(&sound.name);
                commands.spawn((
                    Emitter,
                    Transform::from_translation(Vec3::new(position.x, position.y, 0.0)),
                    AudioPlayer::new(asset_server.load(path)),
                    PlaybackSettings::DESPAWN.with_spatial(true),
                ));
            }
            Audio::Play(sound) => {
                let sound = g.mod_.sound(*sound);
                let path = path.join(&sound.name);
                commands.spawn((
                    AudioPlayer::new(asset_server.load(path)),
                    PlaybackSettings::DESPAWN,
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
