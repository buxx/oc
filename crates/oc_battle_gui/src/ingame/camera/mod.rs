use bevy::prelude::*;

use crate::{ingame::camera::region::Region, states::AppState};

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
            .add_observer(map::on_switch_to_battle_map)
            .add_systems(
                Update,
                (update, region::update_regions)
                    .run_if(in_state(AppState::InGame))
                    .after(move_::move_),
            )
            .add_systems(Update, (move_::move_,).run_if(in_state(AppState::InGame)));
    }
}

#[derive(Debug, Default, Resource)]
pub struct State {
    pub center: Option<Vec2>,
    pub cursor: Option<Vec2>,
    pub regions: Option<Vec<Region>>,
    pub focus: Focus,
    /// Previous camera translation before focus change
    pub previously: Option<Vec3>,
}

fn update(
    camera: Single<(&Camera, &GlobalTransform)>,
    window: Single<&Window>,
    mut state: ResMut<State>,
) {
    let cursor = window.cursor_position();
    let (camera, transform) = *camera;
    let width = window.resolution.width();
    let height = window.resolution.height();
    let center = Vec2::new(width / 2., height / 2.);
    let Ok(center) = camera.viewport_to_world_2d(transform, center) else {
        return;
    };

    state.center = Some(center);
    state.cursor = cursor;
}
