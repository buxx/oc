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

use crate::ingame::behavior::SquadOrder;
use crate::ingame::input::left_click::LeftClick;
use crate::ingame::input::left_click::LeftClickMode;
use crate::ingame::input::left_click::SetLeftClick;
use crate::ingame::path::ComputeDisplayPaths;
use crate::ingame::path::SpawnPathProfile;
use crate::ingame::path::SpawnPathProfileKey;
use crate::network::output::ToServerEvent;
use crate::states::GameConfig;
use crate::utils::drag::Phantom;

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
    drag: Query<&Phantom>,
    markers: Query<&SquadOrder>,
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
    // TODO: maybe too CPU consumer (compute it not at each frames ?)
    show(
        w,
        point,
        order,
        &mut commands,
        &mode,
        &ingame,
        &world,
        &drag,
        &markers,
    );
}

fn show(
    w: &WorldConfig,
    point: WorldVec2,
    order: &OrderType,
    commands: &mut Commands,
    mode: &LeftClick,
    ingame: &crate::ingame::state::State,
    world: &crate::world::World,
    drag: &Query<&Phantom>,
    markers: &Query<&SquadOrder>,
) {
    tracing::trace!(name="ingame-input-left_click-show-order", mode=?mode.0, point=?point);

    match order {
        OrderType::Idle => {}
        OrderType::MoveTo => {
            let spawns = path_profiles(w, point, mode, ingame, world, &drag, &markers);
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
    drag: &Query<&Phantom>,
    markers: &Query<&SquadOrder>,
) -> Vec<SpawnPathProfile> {
    let mut spawns = vec![];
    // Spawns points from selected squads
    if drag.is_empty() {
        spawns.extend(ingame.selected_squads().iter().filter_map(|i| {
            tracing::trace!(name="ingame-input-left_click-show-order-squad", mode=?mode.0, point=?point, squad=?i);
            let pending: Vec<WorldVec2> = ingame.pending_orders().iter().filter_map(|o| o.position()).collect();
            let points = [pending, vec![point]].concat();

            let squad = world.squad(*i)?;
            let leader = world.get_individual(squad.leader())?;
            paths_from(w, *i, points, leader.position.into())
        }).flatten().collect::<Vec<_>>());
    }

    // Spawns points from dragged order markers
    spawns.extend(
        drag.iter()
            .filter_map(|dragged| {
                let mut profiles = vec![];

                let marker = markers.get(dragged.0).ok()?;
                let SquadOrder(squad, index) = marker;
                let i = *squad;
                let squad = world.squad(*squad)?;
                let orders = &squad.orders;
                if orders.is_empty() {
                    return None;
                }

                // Dragged marker is the first marker of squad markers
                if index.0 as usize == orders.len() - 1 {
                    let leader = world.get_individual(squad.leader())?;
                    let paths = paths_from(w, i, vec![point], leader.position.into());
                    profiles.extend(paths.unwrap_or_default())
                // If not, there is a marker before it
                } else {
                    let index = index.0 + 1;
                    let mut orders = orders.iter().rev();
                    if let Some(order) = orders.nth(index as usize) {
                        if let Some(position) = order.position() {
                            let paths = paths_from(w, i, vec![point], position);
                            profiles.extend(paths.unwrap_or_default())
                        }
                    }
                }

                // There is another marker after it
                if index.0 != 0 {
                    let index = index.0 - 1;
                    let mut orders = orders.iter().rev();
                    if let Some(order) = orders.nth(index as usize) {
                        if let Some(position) = order.position() {
                            let paths = paths_from(w, i, vec![point], position);
                            profiles.extend(paths.unwrap_or_default())
                        }
                    }
                }

                Some(profiles)
            })
            .flatten()
            .collect::<Vec<_>>(),
    );

    spawns
}

fn paths_from(
    w: &WorldConfig,
    i: oc_individual::squad::SquadIndex,
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
                    i,
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
    _buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut ingame: ResMut<crate::ingame::state::State>,
    camera: Single<(&Camera, &GlobalTransform)>,
) {
    let_some!(g = &g.0, return);
    let (camera, transform) = *camera;
    // FIXME BS NOW: refacto key42
    let_some!(point = click.hit.position, return);
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
