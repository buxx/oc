use bevy::{prelude::*, window::WindowResized};

use crate::{
    ingame::{
        camera::{
            move_::{MovedBattleCamera, UpdateVisibleBattleSquare},
            region::Region,
        },
        input::map::SwitchToWorldMap,
        world::{DespawnWorldMapBackground, SpawnWorldMapBackground},
    },
    states::{AppState, InGameState},
};

#[cfg(feature = "debug")]
use crate::ingame::region::debug::{DespawnRegionWireFrameDebug, SpawnRegionWireFrameDebug};

#[cfg(feature = "debug")]
pub mod debug;
pub mod map;
pub mod move_;
pub mod region;

pub struct CameraPlugin;

#[derive(Debug, Default)]
pub enum Focus {
    #[default]
    Battle,
    World,
}

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<State>()
            .add_observer(map::on_switch_to_world_map)
            .add_observer(map::on_save_current_window_center_as_battle_center)
            .add_observer(map::on_switch_to_battle_map)
            .add_observer(region::on_update_regions)
            .add_observer(move_::on_moved_battle_camera)
            .add_systems(OnEnter(AppState::InGame), init)
            .add_systems(
                Update,
                (update,)
                    .run_if(in_state(AppState::InGame))
                    .after(move_::move_battle),
            )
            .add_systems(
                Update,
                (move_::move_battle,)
                    .run_if(in_state(AppState::InGame))
                    .run_if(in_state(InGameState::Battle)),
            )
            .add_systems(
                Update,
                (move_::move_in_world_map,)
                    .run_if(in_state(AppState::InGame))
                    .run_if(in_state(InGameState::World)),
            )
            .add_systems(
                Update,
                (on_window_resize_world_map,)
                    .run_if(in_state(AppState::InGame))
                    .run_if(in_state(InGameState::World)),
            );

        #[cfg(feature = "debug")]
        app.add_systems(OnEnter(AppState::InGame), debug::world::setup)
            .add_systems(
                Update,
                debug::world::cursor
                    .run_if(in_state(AppState::InGame))
                    .run_if(in_state(InGameState::World)),
            );
    }
}

#[derive(Debug, Default, Resource)]
pub struct State {
    pub cursor: Option<Vec2>,
    pub regions: Option<Vec<Region>>,
    pub focus: Focus,
    /// Previous camera translation before focus change
    pub previously: Option<Vec3>,
}

fn init(mut commands: Commands) {
    commands.trigger(MovedBattleCamera)
}

fn update(window: Single<&Window>, mut state: ResMut<State>) {
    state.cursor = window.cursor_position();
}

fn on_window_resize_world_map(
    mut commands: Commands,
    resize_reader: MessageReader<WindowResized>,
    state: ResMut<State>,
) {
    if !resize_reader.is_empty() {
        tracing::debug!("Window resized");

        // Ensure positionning on world elements is done with correct window size
        commands.trigger(SwitchToWorldMap);
        commands.trigger(DespawnWorldMapBackground);
        commands.trigger(SpawnWorldMapBackground);

        if let Some(center) = state.previously {
            commands.trigger(UpdateVisibleBattleSquare(Vec2::new(center.x, center.y)));
        }

        #[cfg(feature = "debug")]
        {
            // Region wireframes on world map need to be sapwn again because depends on window size
            static EMPTY: Vec<Region> = vec![];
            for region in state.regions.as_ref().unwrap_or(&EMPTY) {
                commands.trigger(DespawnRegionWireFrameDebug(region.0));
                commands.trigger(SpawnRegionWireFrameDebug(region.0));
            }
        }
    }
}
