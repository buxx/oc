use bevy::prelude::*;
use oc_network::ToServer;
use oc_root::Wcfg;
use oc_utils::{let_ok, let_some};
use oc_world::resume::WorldResume;

use crate::{
    ingame::{
        behavior::BehaviorPlugin,
        draw::world::WorldMapDisplay,
        height::HeightPlugin,
        individual::IndividualPlugin,
        input::InputPlugin,
        lov::LovPlugin,
        path::{DisplayPaths, PathGizmos, draw, on_compute_display_paths},
        projectile::ProjectilePlugin,
        region::on_listening_region,
        squad::SquadPlugin,
        world::{
            on_adjust_minimap, on_despawn_world_map_background, on_spawn_minimap,
            on_spawn_visible_battle_square, on_spawn_world_map_background, on_update_battle_square,
        },
    },
    network::{self, output::ToServerEvent},
    setup::spawn_camera2d,
    states::AppState,
    world::WorldPlugin,
};

pub mod behavior;
pub mod camera;
#[cfg(feature = "debug")]
pub mod debug;
pub mod draw;
pub mod height;
pub mod individual;
pub mod init;
pub mod input;
pub mod lov;
pub mod path;
pub mod physics;
pub mod projectile;
pub mod region;
pub mod squad;
pub mod state;
pub mod world;

pub struct IngamePlugin;

#[derive(Debug, Event)]
pub struct FirstIngameEnter;

#[derive(Debug, Event)]
pub struct GameConfigReceived;

#[derive(Debug, Event, Deref)]
pub struct WorldResumeEvent(pub WorldResume);

#[derive(Debug, Event)]
pub struct SwitchToWorldMap;

#[derive(Debug, Event)]
pub struct SwitchToBattleMap;

#[derive(Debug, Event)]
pub struct QuitHeightMap;

#[derive(Debug, Event)]
pub struct SwitchToHeightMap;

#[derive(Debug, Event)]
pub struct RestoreBattleCenter;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, States)]
pub enum InGameState {
    #[default]
    Battle,
    Height,
    World,
}

impl Plugin for IngamePlugin {
    fn build(&self, app: &mut App) {
        app.init_gizmo_group::<PathGizmos>()
            .add_plugins(InputPlugin)
            .add_plugins(WorldPlugin)
            .add_plugins(HeightPlugin)
            .add_plugins(IndividualPlugin)
            .add_plugins(SquadPlugin)
            .add_plugins(ProjectilePlugin)
            .add_plugins(BehaviorPlugin)
            .add_plugins(LovPlugin)
            .init_resource::<DisplayPaths>()
            .add_observer(on_game_config_received)
            .add_observer(on_update_battle_square)
            .add_observer(on_spawn_minimap)
            .add_observer(on_adjust_minimap)
            .add_observer(on_spawn_visible_battle_square)
            .add_observer(on_spawn_world_map_background)
            .add_observer(on_despawn_world_map_background)
            .add_observer(on_listening_region)
            .add_observer(projectile::on_forgot_projectile)
            .add_observer(on_switch_to_world_map)
            .add_observer(on_switch_to_battle_map)
            .add_observer(on_switch_to_height_map)
            .add_observer(on_quit_height_map)
            .add_observer(on_restore_battle_center)
            .add_observer(on_compute_display_paths)
            // .add_observer(on_forgotten_region)
            .add_observer(physics::on_physics_event)
            // TODO: despawn entities on OnExit(AppState::InGame)
            .add_systems(Startup, (path::setup,))
            .add_systems(
                OnEnter(AppState::InGame),
                (init::init, init::refresh, init::spawn_world_map),
            )
            .add_systems(
                Update,
                (path::draw,)
                    .run_if(in_state(AppState::InGame))
                    .run_if(in_state(InGameState::Battle)),
            );

        #[cfg(feature = "debug")]
        app.add_observer(region::debug::on_listening_region)
            .add_observer(region::debug::on_spawn_region_wire_frame_debug)
            .add_observer(region::debug::on_forgotten_region)
            .add_observer(region::debug::on_despawn_region_wire_frame_debug);
        // .add_observer(init::on_first_ingame_enter)
    }
}

fn on_restore_battle_center(
    _: On<RestoreBattleCenter>,
    state: Res<camera::State>,
    mut camera: Single<&mut Transform, With<Camera2d>>,
) {
    let_some!(previously = state.previously, return);

    tracing::debug!("Restore battle center ({previously:?})");
    camera.translation.x = previously.x;
    camera.translation.y = previously.y;
    camera.translation.z = previously.z;
}

pub fn on_switch_to_battle_map(
    _: On<SwitchToBattleMap>,
    mut ingame: ResMut<NextState<InGameState>>,
    mut state: ResMut<camera::State>,
) {
    tracing::debug!("Set camera focus on Battle");
    state.focus = camera::Focus::Battle;

    tracing::debug!("Set game state to battle");
    *ingame = NextState::Pending(InGameState::Battle);
}

// TODO: rethink order of things to remove camera2d only when height map ready (to instant change things)
pub fn on_switch_to_height_map(
    _: On<SwitchToHeightMap>,
    mut commands: Commands,
    camera2d: Single<Entity, With<Camera2d>>,
    camera: Single<(&Camera, &GlobalTransform)>,
    window: Single<&Window>,
    mut ingame: ResMut<NextState<InGameState>>,
    mut state: ResMut<camera::State>,
) {
    tracing::debug!("Switch to height map");

    let (camera, transform) = *camera;
    let width = window.resolution.width();
    let height = window.resolution.height();
    let center = Vec2::new(width / 2., height / 2.);
    let center = camera.viewport_to_world_2d(transform, center);
    let_ok!(center = center, return);

    commands.entity(*camera2d).despawn();
    *ingame = NextState::Pending(InGameState::Height);
    height::setup_camera3d(&mut commands, &center);
    height::setup_light3d(&mut commands);

    tracing::debug!("Set game state to height");
    *ingame = NextState::Pending(InGameState::Height);

    tracing::debug!("Set camera focus on Height");
    state.focus = camera::Focus::Height;

    tracing::debug!("Trigger spawn height (center={center:?}");
    commands.trigger(height::Spawn(center));
}

pub fn on_quit_height_map(
    _: On<QuitHeightMap>,
    mut commands: Commands,
    camera3d: Single<Entity, With<Camera3d>>,
    light3d: Single<Entity, With<DirectionalLight>>,
) {
    tracing::debug!("Despawn camera3d");
    commands.entity(*camera3d).despawn();

    tracing::debug!("Despawn light3d");
    commands.entity(*light3d).despawn();

    tracing::debug!("Spawn camera2d");
    spawn_camera2d(&mut commands);
}

pub fn on_switch_to_world_map(
    _: On<SwitchToWorldMap>,
    w: Res<Wcfg>,
    mut camera: Single<&mut Transform, With<Camera2d>>,
    mut state: ResMut<camera::State>,
    window: Single<&Window>,
    mut ingame: ResMut<NextState<InGameState>>,
) {
    tracing::debug!("Switch to world map");
    let_some!(w = &w.0, return);

    let display = WorldMapDisplay::from_env(w, window.size());
    tracing::debug!("Set camera focus on World");
    state.focus = camera::Focus::World;
    camera.translation.x = display.center.x;
    camera.translation.y = display.center.y;

    tracing::debug!("Set game state to world");
    *ingame = NextState::Pending(InGameState::World);
}

fn on_game_config_received(
    _: On<GameConfigReceived>,
    mut commands: Commands,
    network_state: Res<network::state::State>,
) {
    let_some!(identity = &network_state.identity, return);
    commands.trigger(ToServerEvent(ToServer::RequestInit(identity.clone())));
}
