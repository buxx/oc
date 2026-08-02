use bevy::prelude::*;
use oc_geo::tile::TileXy;
use oc_geo::tile::WorldTileIndex;
use oc_individual::order::OrderType;
use oc_root::Wcfg;
use oc_root::WcfgFrom;
use oc_root::WorldConfig;
use oc_root::y::Y;
use oc_utils::let_ok;
use oc_utils::let_some;
use oc_utils::return_if;

use crate::ingame::input::left_click::LeftClick;
use crate::ingame::input::left_click::LeftClickMode;
use crate::ingame::input::left_click::SetLeftClick;
use crate::ingame::path::ComputeDisplayPaths;
use crate::ingame::path::SpawnPathProfile;
use crate::ingame::path::SpawnPathProfileKey;
use crate::window::PointerInWindow;

pub fn system(
    mut commands: Commands,
    w: Res<Wcfg>,
    ignore: Res<PointerInWindow>, // TODO: use state ?
    window: Single<&Window>,
    camera: Single<(&Camera, &GlobalTransform)>,
    buttons: Res<ButtonInput<MouseButton>>,
    mode: Res<LeftClick>,
    keys: Res<ButtonInput<KeyCode>>,
    world: Res<crate::world::World>,
    ingame: ResMut<crate::ingame::state::State>,
) {
    if ignore.0 {
        return;
    }
    let_some!(w = &w.0, return);
    let_some!(cursor = window.cursor_position(), return);
    let (camera, transform) = *camera;
    let point = camera.viewport_to_world_2d(transform, cursor);
    let_ok!(point = point, return);

    let LeftClickMode::Order(order) = &mode.0 else {
        return;
    };

    return_if!(maybe_cancel(&mut commands, &buttons, &keys));
    show(w, point, order, &mut commands, &mode, &ingame, &world);
}

fn show(
    w: &WorldConfig,
    point: Vec2,
    order: &OrderType,
    commands: &mut Commands,
    mode: &LeftClick,
    ingame: &crate::ingame::state::State,
    world: &crate::world::World,
) {
    tracing::trace!(name="ingame-input-left_click-show-order", mode=?mode.0, point=?point);

    match order {
        OrderType::Idle => {}
        OrderType::MoveTo => {
            let spawns = ingame.selected_squads().iter().filter_map(|i| {
                tracing::trace!(name="ingame-input-left_click-show-order-squad", mode=?mode.0, point=?point, squad=?i);
                let squad = world.squad(i)?;
                let leader = world.get_individual(squad.leader())?;
                let start = Vec2::new(leader.position[0], leader.position[1]);
                let start = start.to_gui_y(w);
                let end = point;
                let start_tile = TileXy::from_([start.x, start.y], w);
                let start_tile = WorldTileIndex::from_(start_tile, w);
                let key = SpawnPathProfileKey::Squad{ i: *i, start: start_tile, end };
                Some(SpawnPathProfile { key, start, end })
            }).collect::<Vec<SpawnPathProfile>>();
            commands.trigger(ComputeDisplayPaths(spawns));
        }
    }
}

fn maybe_cancel(
    commands: &mut Commands,
    buttons: &ButtonInput<MouseButton>,
    keys: &ButtonInput<KeyCode>,
) -> bool {
    if keys.just_pressed(KeyCode::Escape) || buttons.just_pressed(MouseButton::Middle) {
        tracing::trace!(name = "ingame-input-left-click-order-abort");
        commands.trigger(ComputeDisplayPaths(vec![]));
        commands.trigger(SetLeftClick(LeftClickMode::Select));
        return true;
    }
    false
}
