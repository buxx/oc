use std::time::Duration;

use bevy::{prelude::*, sprite::Anchor};
use oc_physics::update::bevy::Position;
use oc_root::{
    WcfgInto, WorldConfig,
    files::FilesAsGui,
    geo::{ScreenVec2, WorldVec2},
};
use oc_utils::let_some;
use oc_world::interior::Interior;

use crate::{
    entity::individual::Side,
    ingame::{draw::Z_INTERIOR, individual::Status},
    network,
    states::AppState,
};

pub struct InteriorPlugin;

impl Plugin for InteriorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            update_interiors_visibility.run_if(in_state(AppState::InGame)),
        );
    }
}

pub fn spawn(
    commands: &mut Commands,
    assets: &AssetServer,
    files: &FilesAsGui,
    w: &WorldConfig,
    interior: Interior,
) {
    let path = files.interior(interior.id);
    // Cropped image starts at top left rect of polygon
    let start = WorldVec2::new(interior.start.x, interior.start.y);
    let position: ScreenVec2 = start.into_(w);
    let position = Vec3::new(position.x, position.y, Z_INTERIOR);

    tracing::trace!(name="spawn-interior", id=interior.id, path=?path);
    commands.spawn((
        Sprite::from_image(assets.load(path)),
        Anchor::TOP_LEFT,
        Transform::from_translation(position),
        Visibility::Hidden,
        interior,
    ));
}

fn update_interiors_visibility(
    time: Res<Time>,
    g: Res<crate::states::GameConfig>,
    network: Res<network::state::State>,
    mut timer: Local<Option<Timer>>,
    individuals: Query<(&Position, &Side, &Status)>,
    mut interiors: Query<(&Interior, &mut Visibility)>,
) {
    let_some!(g = &g.0, return);
    let_some!(identity = &network.identity, return);

    let period = Duration::from_secs_f32(g.w.interiors_tick().period());
    let timer = timer.get_or_insert_with(|| Timer::new(period, TimerMode::Repeating));
    if !timer.tick(time.delta()).just_finished() {
        return;
    }

    let positions: Vec<_> = individuals
        .iter()
        .filter(|(_, side, status)| side.0 == identity.side && status.0.can_step())
        .map(|(position, _, _)| (position.0.x, position.0.y))
        .collect();

    for (interior, mut visibility) in &mut interiors {
        let displayed = positions.iter().any(|(x, y)| interior.contains(*x, *y));
        match displayed {
            true => *visibility = Visibility::Visible,
            false => *visibility = Visibility::Hidden,
        };
    }
}
