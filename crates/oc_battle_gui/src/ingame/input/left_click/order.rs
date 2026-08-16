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
use oc_root::y::V;
use oc_root::y::Y;
use oc_utils::d2::Direction;
use oc_utils::let_ok;
use oc_utils::let_some;
use oc_utils::return_if;

use crate::cursor_to;
use crate::ingame::InGameState;
use crate::ingame::behavior::DirectionSquadOrder;
use crate::ingame::behavior::PositionSquadOrder;
use crate::ingame::draw::UI_FILE;
use crate::ingame::draw::Z_SQUAD_ORDER;
use crate::ingame::input::left_click::LeftClick;
use crate::ingame::input::left_click::LeftClickMode;
use crate::ingame::input::left_click::LeftClickModeType;
use crate::ingame::input::left_click::SetLeftClick;
use crate::ingame::path::ComputeDisplayPaths;
use crate::ingame::path::SpawnPathProfile;
use crate::ingame::path::SpawnPathProfileKey;
use crate::network::output::ToServerEvent;
use crate::sprites::SpriteRect;
use crate::sprites::order::SquadOrderSprite;
use crate::states::AppState;
use crate::states::GameConfig;
use crate::utils::drag;
use crate::utils::drag::Dragged;
use crate::utils::drag::Phantom;
use crate::utils::selected::Selected;

/// Marker which indicate order is pending (player prepare to give it)
#[derive(Debug, Clone, Component)]
pub struct PendingOrder;

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
    position_markers: Query<&PositionSquadOrder>,
    mut direction_markers: Query<(&DirectionSquadOrder, &mut Transform), With<PendingOrder>>,
) {
    let_some!(w = &w.0, return);
    let_some!(cursor = window.cursor_position(), return);
    let point = cursor_to!(cursor, camera, w, WorldVec2);

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
        &position_markers,
        &mut direction_markers,
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
    position_markers: &Query<&PositionSquadOrder>,
    direction_markers: &mut Query<(&DirectionSquadOrder, &mut Transform), With<PendingOrder>>,
) {
    tracing::trace!(name="ingame-input-left_click-show-order", mode=?mode.0, point=?point);

    match order {
        OrderType::Idle | OrderType::Defend | OrderType::Hide => {
            rotate_direction_markers(point, world, direction_markers);
        }
        OrderType::MoveTo | OrderType::MoveFastTo | OrderType::SneakTo => {
            let spawns = path_profiles(w, point, mode, ingame, world, &drag, &position_markers);
            commands.trigger(ComputeDisplayPaths(spawns));
        }
    }
}

fn rotate_direction_markers(
    point: WorldVec2,
    world: &crate::world::World,
    direction_markers: &mut Query<(&DirectionSquadOrder, &mut Transform), With<PendingOrder>>,
) {
    for (order, mut transform) in direction_markers.iter_mut() {
        let_some!(squad = world.squad(order.squad()), continue);
        let reference = squad.position;
        let direction = Direction::from_points2d(reference.into(), point.into());
        *transform = transform.with_rotation(direction.bquat(V::Gui));
    }
}

fn path_profiles(
    w: &WorldConfig,
    point: WorldVec2,
    mode: &LeftClick,
    ingame: &crate::ingame::state::State,
    world: &crate::world::World,
    drag: &Query<&Phantom>,
    markers: &Query<&PositionSquadOrder>,
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
                let PositionSquadOrder(squad, index) = marker;
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
    left_click: Res<LeftClick>,
    world: Res<crate::world::World>,
) {
    let_some!(g = &g.0, return);
    let_some!(point = click.hit.position, return);
    let point = Vec2::new(point.x, point.y);
    let point = cursor_to!(point, camera, &g.w, WorldVec2);

    match click.button {
        PointerButton::Primary => {
            let adding = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
            let LeftClickMode::Order(order_type) = left_click.0 else {
                return;
            };
            tracing::trace!(name = "ingame-input-left-click-order-action");

            if !adding {
                give_orders(&mut commands, &ingame, &world, point, Some(order_type));
                cancel(&mut commands, &mut ingame);
            } else {
                if let Some(order) = match order_type {
                    OrderType::MoveTo => Some(Order::MoveTo(point)),
                    OrderType::MoveFastTo => Some(Order::MoveFastTo(point)),
                    OrderType::SneakTo => Some(Order::SneakTo(point)),
                    OrderType::Idle | OrderType::Defend | OrderType::Hide => None,
                } {
                    ingame.push_pending_orders(order);
                }
            }
        }
        PointerButton::Secondary => {
            give_orders(&mut commands, &ingame, &world, point, None);
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
    world: &crate::world::World,
    point: WorldVec2,
    order_type: Option<OrderType>,
) {
    for squad in ingame.selected_squads() {
        let mut orders = ingame.pending_orders().to_vec();
        if let Some(order_type) = &order_type {
            let_some!(squad_ = world.squad(*squad), continue);
            // FIXME: With multiple squad, need decal a little (distance from each others ?)
            let order = order_type.into_order(point, squad_.position);
            orders.push(order.clone());
        }
        let set_orders = oc_network::SquadMessage::SetOrders(orders.clone());
        commands.trigger(ToServerEvent(ToServer::Squad(*squad, set_orders)));
    }
}

/// React to left click mode change to defend/hide order and spawn position order marker on selected squads
pub fn on_set_left_click(
    event: On<SetLeftClick>,
    g: Res<GameConfig>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    state: Res<crate::ingame::state::State>,
    world: Res<crate::world::World>,
) {
    let_some!(g = &g.0, return);

    for squad in state.selected_squads() {
        if let Some((component, sprite)) = match event.0 {
            LeftClickMode::Order(order) => match order {
                OrderType::Defend => Some((
                    DirectionSquadOrder::Defend(*squad),
                    SquadOrderSprite::Defend,
                )),
                OrderType::Hide => {
                    Some((DirectionSquadOrder::Hide(*squad), SquadOrderSprite::Hide))
                }
                OrderType::Idle
                | OrderType::MoveTo
                | OrderType::MoveFastTo
                | OrderType::SneakTo => None,
            },
            LeftClickMode::Select | LeftClickMode::LineOfView(_) => None,
            #[cfg(feature = "debug")]
            LeftClickMode::SpawnProjectile(_) => None,
        } {
            let_some!(squad = world.squad(*squad), continue);

            let point = squad.position;
            let image = asset_server.load(UI_FILE);
            let rect = sprite.rect();
            let sprite = Sprite {
                image,
                rect: Some(rect),
                ..default()
            };

            // FIXME BS NOW: when squad leader position update, update direction order too
            tracing::debug!("Spawn direction order {component:?} at {point:?}");
            commands
                .spawn((
                    component.clone(),
                    PendingOrder,
                    sprite,
                    Transform::from_xyz(point.x, point.y.to_gui_y(&g.w), Z_SQUAD_ORDER),
                    Pickable::default(),
                    Selected::default(),
                    Dragged::<DirectionSquadOrder>::default(),
                ))
                .observe(
                    drag::on_drag_start::<DirectionSquadOrder>
                        .run_if(in_state(AppState::InGame))
                        .run_if(in_state(InGameState::Battle))
                        .run_if(in_state(LeftClickModeType::Select)),
                );
        }
    }
}
