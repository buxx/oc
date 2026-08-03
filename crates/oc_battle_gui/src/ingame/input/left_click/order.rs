use bevy::prelude::*;
use oc_geo::tile::TileXy;
use oc_geo::tile::WorldTileIndex;
use oc_individual::order::Order;
use oc_individual::order::OrderType;
use oc_network::ToServer;
use oc_root::Wcfg;
use oc_root::WcfgFrom;
use oc_root::WorldConfig;
use oc_root::geo::WorldPoint2d;
use oc_root::y::Y;
use oc_utils::d2::Position;
use oc_utils::let_ok;
use oc_utils::let_some;
use oc_utils::return_if;

use crate::ingame::input::left_click::LeftClick;
use crate::ingame::input::left_click::LeftClickMode;
use crate::ingame::input::left_click::SetLeftClick;
use crate::ingame::path::ComputeDisplayPaths;
use crate::ingame::path::SpawnPathProfile;
use crate::ingame::path::SpawnPathProfileKey;
use crate::network::output::ToServerEvent;
use crate::window::PointerInWindow;

#[derive(Debug, Clone, Copy, Resource, Deref, DerefMut, Default)]
pub struct OnGoing(pub bool);

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
    mut ongoing: ResMut<OnGoing>,
) {
    if ignore.0 {
        return;
    }
    let_some!(w = &w.0, return);
    let_some!(cursor = window.cursor_position(), return);
    let (camera, transform) = *camera;
    let point = camera.viewport_to_world_2d(transform, cursor);
    let_ok!(point = point, return);
    let point = WorldPoint2d::from_(point, w);

    let LeftClickMode::Order(order) = &mode.0 else {
        return;
    };

    return_if!(maybe_cancel(&mut commands, &buttons, &keys, &mut ongoing));
    show(w, point, order, &mut commands, &mode, &ingame, &world);
    action(&mut ongoing, point, &buttons, &mut commands, &ingame);
    ongoing.0 = true;
}

fn show(
    w: &WorldConfig,
    point: WorldPoint2d,
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
                // FIXME BS NOW: WorldPoint2d inside individual
                let start = WorldPoint2d::new(leader.position[0], leader.position[1]);
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
    ongoing: &mut OnGoing,
) -> bool {
    if keys.just_pressed(KeyCode::Escape) || buttons.just_pressed(MouseButton::Middle) {
        tracing::trace!(name = "ingame-input-left-click-order-cancel");
        cancel(commands, ongoing);
        return true;
    }
    false
}

fn cancel(commands: &mut Commands, ongoing: &mut OnGoing) {
    commands.trigger(ComputeDisplayPaths(vec![]));
    commands.trigger(SetLeftClick(LeftClickMode::Select));
    ongoing.0 = false;
}

fn action(
    ongoing: &mut OnGoing,
    point: WorldPoint2d,
    buttons: &ButtonInput<MouseButton>,
    commands: &mut Commands,
    ingame: &crate::ingame::state::State,
) {
    // FIXME BS NOW: multi step
    if ongoing.0 && buttons.just_pressed(MouseButton::Left) {
        tracing::trace!(name = "ingame-input-left-click-order-action");

        cancel(commands, ongoing);

        for squad in ingame.selected_squads() {
            let orders = vec![Order::MoveTo(Position::new(point.x, point.y))];
            let set_orders = oc_network::SquadMessage::SetOrders(orders);
            commands.trigger(ToServerEvent(ToServer::Squad(*squad, set_orders)));
        }
    }
}
