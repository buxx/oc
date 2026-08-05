use bevy::prelude::*;
use oc_geo::tile::TileXy;
use oc_geo::tile::WorldTileIndex;
use oc_individual::order::Order;
use oc_individual::order::OrderType;
use oc_network::ToServer;
use oc_root::Wcfg;
use oc_root::WcfgFrom;
use oc_root::WorldConfig;
use oc_root::geo::WorldVec2;
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

#[derive(Debug, Clone, Copy, Resource, Deref, DerefMut, Default)]
pub struct OnGoing(pub bool);

pub fn system(
    mut commands: Commands,
    w: Res<Wcfg>,
    window: Single<&Window>,
    camera: Single<(&Camera, &GlobalTransform)>,
    buttons: Res<ButtonInput<MouseButton>>,
    mode: Res<LeftClick>,
    keys: Res<ButtonInput<KeyCode>>,
    world: Res<crate::world::World>,
    mut ingame: ResMut<crate::ingame::state::State>,
    mut ongoing: ResMut<OnGoing>,
) {
    let_some!(w = &w.0, return);
    let_some!(cursor = window.cursor_position(), return);
    let (camera, transform) = *camera;
    let point = camera.viewport_to_world_2d(transform, cursor);
    let_ok!(point = point, return);
    let point = WorldVec2::from_(point, w);

    let LeftClickMode::Order(order) = &mode.0 else {
        return;
    };

    return_if!(maybe_cancel(
        &mut commands,
        &buttons,
        &keys,
        &mut ongoing,
        &mut ingame
    ));
    show(w, point, order, &mut commands, &mode, &ingame, &world);
    action(
        &mut ongoing,
        point,
        &buttons,
        &keys,
        &mut commands,
        &mut ingame,
    );
    ongoing.0 = true;
}

fn show(
    w: &WorldConfig,
    point: WorldVec2,
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
            let spawns = path_profiles(w, point, mode, ingame, world);
            commands.trigger(ComputeDisplayPaths(spawns));
        }
    }
}

fn path_profiles(
    w: &WorldConfig,
    point: WorldVec2,
    mode: &LeftClick,
    ingame: &crate::ingame::state::State,
    world: &crate::world::World,
) -> Vec<SpawnPathProfile> {
    let spawns = ingame.selected_squads().iter().filter_map(|i| {
        tracing::trace!(name="ingame-input-left_click-show-order-squad", mode=?mode.0, point=?point, squad=?i);
        let pending: Vec<WorldVec2> = ingame.pending_orders().iter().filter_map(|o| o.point()).collect();
        let points = [pending, vec![point]].concat();

        let squad = world.squad(*i)?;
        let leader = world.get_individual(squad.leader())?;
        paths_from(w, i, points, leader.position.into())
    }).flatten().collect::<Vec<_>>();
    spawns
}

fn paths_from(
    w: &WorldConfig,
    i: &oc_individual::squad::SquadIndex,
    points: Vec<WorldVec2>,
    start: WorldVec2,
) -> Option<Vec<SpawnPathProfile>> {
    let mut start = start;
    Some(
        points
            .into_iter()
            .map(|point| {
                let end: WorldVec2 = point;
                let start_tile = TileXy::from_([start.x, start.y], w);
                let start_tile = WorldTileIndex::from_(start_tile, w);
                let key = SpawnPathProfileKey::Squad {
                    i: *i,
                    start: start_tile,
                    end,
                };
                let profile = SpawnPathProfile { key, start, end };
                start = end;
                profile
            })
            .collect::<Vec<_>>(),
    )
}

fn maybe_cancel(
    commands: &mut Commands,
    buttons: &ButtonInput<MouseButton>,
    keys: &ButtonInput<KeyCode>,
    ongoing: &mut OnGoing,
    ingame: &mut crate::ingame::state::State,
) -> bool {
    if keys.just_pressed(KeyCode::Escape) || buttons.just_pressed(MouseButton::Middle) {
        tracing::trace!(name = "ingame-input-left-click-order-cancel");
        cancel(commands, ongoing, ingame);
        return true;
    }
    false
}

fn cancel(
    commands: &mut Commands,
    ongoing: &mut OnGoing,
    ingame: &mut crate::ingame::state::State,
) {
    commands.trigger(ComputeDisplayPaths(vec![]));
    commands.trigger(SetLeftClick(LeftClickMode::Select));
    ongoing.0 = false;
    ingame.clear_pending_orders();
}

// FIXME BS NOW: when click to set orders, do not clear selection (jouer avec les state LeftClick::Order ?)
fn action(
    ongoing: &mut OnGoing,
    point: WorldVec2,
    buttons: &ButtonInput<MouseButton>,
    keys: &ButtonInput<KeyCode>,
    commands: &mut Commands,
    ingame: &mut crate::ingame::state::State,
) {
    if ongoing.0
        && (buttons.just_pressed(MouseButton::Left) || buttons.just_pressed(MouseButton::Middle))
    {
        let adding = keys.pressed(KeyCode::ControlLeft)
            || keys.pressed(KeyCode::ControlRight)
            || buttons.just_pressed(MouseButton::Middle);
        let mut orders = ingame.pending_orders().to_vec();
        // TODO: When multiple squad, need decal a little (distance from each others ?)
        let order = Order::MoveTo(point);

        tracing::trace!(name = "ingame-input-left-click-order-action");

        if !adding {
            cancel(commands, ongoing, ingame);

            for squad in ingame.selected_squads() {
                orders.push(order.clone());
                let set_orders = oc_network::SquadMessage::SetOrders(orders.clone());
                commands.trigger(ToServerEvent(ToServer::Squad(*squad, set_orders)));
            }
        } else {
            ingame.push_pending_orders(order);
        }
    }
}
