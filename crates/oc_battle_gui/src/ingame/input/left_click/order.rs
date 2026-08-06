use bevy::prelude::*;
use oc_geo::tile::TileXy;
use oc_geo::tile::WorldTileIndex;
use oc_individual::order::Order;
use oc_individual::order::OrderType;
use oc_network::ToServer;
use oc_root::Wcfg;
use oc_root::WcfgFrom;
use oc_root::WorldConfig;
use oc_root::geo::ScreenVec2;
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
use crate::states::GameConfig;

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

    return_if!(maybe_cancel(&mut commands, &buttons, &keys, &mut ingame));
    show(w, point, order, &mut commands, &mode, &ingame, &world);
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
    _buttons: &ButtonInput<MouseButton>,
    keys: &ButtonInput<KeyCode>,
    ingame: &mut crate::ingame::state::State,
) -> bool {
    if keys.just_pressed(KeyCode::Escape) {
        tracing::trace!(name = "ingame-input-left-click-order-cancel");
        cancel(commands, ingame);
        return true;
    }
    false
}

fn cancel(commands: &mut Commands, ingame: &mut crate::ingame::state::State) {
    commands.trigger(ComputeDisplayPaths(vec![]));
    commands.trigger(SetLeftClick(LeftClickMode::Select));
    ingame.clear_pending_orders();
}

pub fn on_click(
    mut click: On<Pointer<Click>>,
    g: Res<GameConfig>,
    mut commands: Commands,
    camera: Single<(&Camera, &GlobalTransform)>,
    _buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut ingame: ResMut<crate::ingame::state::State>,
) {
    let_some!(g = &g.0, return);
    let_some!(point = click.hit.position, return);
    let (camera, transform) = *camera;
    let point = Vec2::new(point.x, point.y);
    let point = camera.viewport_to_world_2d(transform, point);
    let_ok!(point = point, return);
    let point = ScreenVec2::new(point.x, point.y);
    let point = WorldVec2::from_(point, &g.w);

    match click.button {
        PointerButton::Primary => {
            let adding = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
            // TODO: When multiple squad, need decal a little (distance from each others ?)
            let order = Order::MoveTo(point);

            tracing::trace!(name = "ingame-input-left-click-order-action");

            if !adding {
                give_orders(&mut commands, &ingame, Some(order));
                cancel(&mut commands, &mut ingame);
            } else {
                ingame.push_pending_orders(order);
            }
        }
        PointerButton::Secondary => {
            give_orders(&mut commands, &ingame, None);
            cancel(&mut commands, &mut ingame);
        }
        PointerButton::Middle => {
            ingame.pop_pending_order();
        }
    };

    click.propagate(false);
}

fn give_orders(
    commands: &mut Commands,
    ingame: &crate::ingame::state::State,
    order: Option<Order>,
) {
    for squad in ingame.selected_squads() {
        let mut orders = ingame.pending_orders().to_vec();
        if let Some(order) = &order {
            orders.push(order.clone());
        }
        let set_orders = oc_network::SquadMessage::SetOrders(orders.clone());
        commands.trigger(ToServerEvent(ToServer::Squad(*squad, set_orders)));
    }
}
