use bevy::prelude::*;
use oc_individual::{
    order::{Order, OrderIndex, OrderType},
    squad::SquadIndex,
};
use oc_network::{SquadMessage, ToServer};
use oc_root::{
    geo::WorldVec2,
    y::{V, Y},
};
use oc_utils::{bevy::EntityMapping, collections::InvertedIndex, d2::Direction, let_ok, let_some};
use rustc_hash::FxHashMap;

use crate::{
    ingame::{
        InGameState,
        draw::{self, UI_FILE, Z_SQUAD_ORDER},
        input::left_click::{LeftClickMode, LeftClickModeType, SetLeftClick, order::PendingOrder},
        path::ComputeDisplayPaths,
        region::{ForgottenRegion, ListeningRegion},
    },
    network::output::ToServerEvent,
    sprites::{IntoIndividualSprite, IntoSprite, SpriteRect},
    states::{AppState, GameConfig},
    utils::{
        drag::{self, DragPlugin, Dragged, Dragging, Phantom},
        selected::Selected,
    },
    world::World,
};

pub struct BehaviorPlugin;

#[derive(Debug, Resource, Default, Deref, DerefMut)]
pub struct IndividualOrders(FxHashMap<oc_individual::IndividualIndex, Vec<(Order, Entity)>>);

// FIXME BS NOW: Seems not used, try delete it
#[derive(Debug, Resource, Default, Deref, DerefMut)]
pub struct SquadOrders(FxHashMap<oc_individual::squad::SquadIndex, Vec<(Order, Entity)>>);

#[derive(Debug, Event)]
pub struct RefreshIndividualOrdersEvent(
    pub oc_individual::IndividualIndex,
    pub Vec<oc_individual::order::Order>,
);

#[derive(Debug, Event, PartialEq)]
pub struct RefreshSquadsOrdersEvent(
    pub oc_individual::squad::SquadIndex,
    pub Vec<oc_individual::order::Order>,
);

#[derive(Debug, Event)]
pub struct SpawnIndividualOrder(oc_individual::IndividualIndex, oc_individual::order::Order);

#[derive(Debug, Event)]
pub enum SpawnSquadOrder {
    Position(
        oc_individual::squad::SquadIndex,
        oc_individual::order::OrderIndex,
        oc_individual::order::Order,
    ),
    Direction(
        oc_individual::squad::SquadIndex,
        oc_individual::order::Order,
        bool, // Pending (true when player is giving order)
    ),
}

impl SpawnSquadOrder {
    pub fn squad(&self) -> SquadIndex {
        match self {
            SpawnSquadOrder::Position(squad, _, _) | SpawnSquadOrder::Direction(squad, _, _) => {
                *squad
            }
        }
    }

    pub fn order(&self) -> &Order {
        match self {
            SpawnSquadOrder::Position(_, _, order) | SpawnSquadOrder::Direction(_, order, _) => {
                order
            }
        }
    }
}

#[derive(Debug, Event)]
pub struct SpawnSquadOrders(
    pub oc_individual::squad::SquadIndex,
    pub Vec<oc_individual::order::Order>,
);

#[derive(Debug, Event)]
pub struct DespawnIndividualOrder(
    pub oc_individual::IndividualIndex,
    pub oc_individual::order::Order,
);

#[derive(Debug, Event)]
pub struct DespawnIndividualOrders(pub oc_individual::IndividualIndex);

#[derive(Debug, Event)]
pub struct DespawnSquadOrders(pub oc_individual::squad::SquadIndex);

#[derive(Debug, Event)]
pub struct DespawnSquadOrder(
    pub oc_individual::squad::SquadIndex,
    oc_individual::order::Order,
);

#[derive(Debug, Component)]
pub struct PositionSquadOrder(pub SquadIndex, pub OrderIndex);

impl Dragging for PositionSquadOrder {
    fn spawn(commands: &mut Commands, marker: Phantom) {
        commands.trigger(SpawnSquadOrderMarkerPhantom(marker));
    }

    fn drop(commands: &mut Commands, subject: Entity, point: WorldVec2) {
        commands.trigger(DropSquadOrderMarkerPhantom(subject, point));
    }

    fn visual() -> drag::Visual {
        drag::Visual::Offset
    }
}

#[derive(Debug, Event)]
pub struct SpawnSquadOrderMarkerPhantom(Phantom);

#[derive(Debug, Event)]
pub struct DropSquadOrderMarkerPhantom(Entity, WorldVec2);

#[derive(Debug, Clone, Component)]
pub enum DirectionSquadOrder {
    Defend(SquadIndex),
    Hide(SquadIndex),
}

#[derive(Debug, Event)]
pub struct EnterDragDirectionSquadOrderMarker(Phantom);

#[derive(Debug, Event)]
pub struct UpdateDirectionSquadOrderTarget(Entity, WorldVec2);

impl Dragging for DirectionSquadOrder {
    fn spawn(commands: &mut Commands, marker: Phantom) {
        commands.trigger(EnterDragDirectionSquadOrderMarker(marker));
    }

    fn drop(commands: &mut Commands, subject: Entity, point: WorldVec2) {
        commands.entity(subject).remove::<Phantom>();
        tracing::trace!(name = "ingame-behavior-dragging-direction-squad-order-drop-trigger",);
        commands.trigger(UpdateDirectionSquadOrderTarget(subject, point));
    }

    fn visual() -> drag::Visual {
        drag::Visual::Direction
    }
}

impl DirectionSquadOrder {
    pub fn squad(&self) -> SquadIndex {
        match self {
            DirectionSquadOrder::Defend(squad) | DirectionSquadOrder::Hide(squad) => *squad,
        }
    }
}

pub fn on_spawn_squad_order_marker_phantom(
    event: On<SpawnSquadOrderMarkerPhantom>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    query: Query<&mut PositionSquadOrder>,
    world: Res<crate::world::World>,
) {
    let_ok!(order = query.get(event.0.0), return);
    let (squad, order) = (order.0, order.1);
    let_some!(squad = world.squad(squad), return);
    let_some!(order = squad.orders.get_r(order.0 as usize), return);
    let image = asset_server.load(UI_FILE);
    let rect = order.sprite().rect();
    let marker = event.0;
    let sprite = Sprite {
        image,
        rect: Some(rect),
        ..default()
    };
    commands.spawn((sprite, Transform::default(), marker));
    commands.trigger(SetLeftClick(LeftClickMode::Order(order.order_type())));
}

pub fn on_drop_squad_order_marker_phantom(
    event: On<DropSquadOrderMarkerPhantom>,
    mut commands: Commands,
    query: Query<&mut PositionSquadOrder>,
) {
    let_ok!(order = query.get(event.0), return);
    let (squad, index, position) = (order.0, order.1, event.1);
    let message = SquadMessage::SetPositionOrderPosition(index, position);
    tracing::trace!(name = "ingame-behavior-on-drop-squad-order-marker-phantom", squad=?squad, index=?index, position=?position);
    commands.trigger(ToServerEvent(ToServer::Squad(squad, message)));
    commands.trigger(SetLeftClick(LeftClickMode::Select));
    commands.trigger(ComputeDisplayPaths(vec![]));
}

pub fn on_enter_drag_direction_squad_order_marker(
    event: On<EnterDragDirectionSquadOrderMarker>,
    mut commands: Commands,
    markers: Query<&DirectionSquadOrder>,
) {
    dbg!(0);
    let phantom = event.0;
    let_ok!(order = markers.get(phantom.0), return);
    let order_type = match order {
        DirectionSquadOrder::Defend(_) => OrderType::Defend,
        DirectionSquadOrder::Hide(_) => OrderType::Hide,
    };

    tracing::trace!(name = "ingame-behavior-on-enter-drag-direction-squad-order-marker");
    // The position order become the phantom itself
    commands.entity(phantom.0).insert(phantom);
    // Prevent other system/observer
    commands.trigger(SetLeftClick(LeftClickMode::Order(order_type)));
}

pub fn on_update_direction_squad_order_target(
    event: On<UpdateDirectionSquadOrderTarget>,
    mut commands: Commands,
    query: Query<&mut DirectionSquadOrder>,
    world: Res<crate::world::World>,
) {
    let_ok!(order = query.get(event.0), return);
    let (squad, target) = (order.squad(), event.1);
    let_some!(squad_ = world.squad(order.squad()), return);
    let reference = squad_.position;
    let direction = Direction::from_points2d(reference.into(), target.into());

    let order = match order {
        DirectionSquadOrder::Defend(_) => Order::Defend(direction),
        DirectionSquadOrder::Hide(_) => Order::Hide(direction),
    };

    tracing::trace!(name="ingame-behavior-on-update-direction-squad-order-target", squad=?squad, order=?order);
    let orders = SquadMessage::SetOrders(vec![order]);
    commands.trigger(ToServerEvent(ToServer::Squad(squad, orders)));
    commands.trigger(SetLeftClick(LeftClickMode::Select));
}

impl Plugin for BehaviorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DragPlugin::<PositionSquadOrder>::default())
            .add_plugins(DragPlugin::<DirectionSquadOrder>::default())
            .init_resource::<IndividualOrders>()
            .init_resource::<SquadOrders>()
            .add_observer(on_refresh_individual_orders)
            .add_observer(on_spawn_individual_order)
            .add_observer(on_despawn_individual_order)
            .add_observer(on_despawn_individual_orders)
            .add_observer(on_refresh_squad_orders)
            .add_observer(on_spawn_squad_order)
            .add_observer(on_spawn_squad_orders)
            .add_observer(on_despawn_squad_order)
            .add_observer(on_despawn_squad_orders)
            .add_observer(on_listening_region)
            .add_observer(on_forgotten_region)
            .add_observer(on_spawn_squad_order_marker_phantom)
            .add_observer(on_drop_squad_order_marker_phantom)
            .add_observer(on_enter_drag_direction_squad_order_marker)
            .add_observer(on_update_direction_squad_order_target);
        // FIXME BS NOW: must spawn/despawn according to existing squad order and not only pending;
        // FIXME BS NOW: currently, when set new order, direction squad order entity (visible by sprite) is not despawn
    }
}

fn on_refresh_individual_orders(
    event: On<RefreshIndividualOrdersEvent>,
    orders: Res<IndividualOrders>,
    mut commands: Commands,
) {
    let (i, orders_) = (event.0, &event.1);
    tracing::trace!(name = "ingame-behavior-on-refresh-individual-orders", i=?i, orders=?orders_, x=?orders.0);

    // Search for new ones
    for order in orders_ {
        if orders
            .get(&i)
            .and_then(|orders| orders.iter().find(|(o, _)| o == order))
            .is_none()
        {
            tracing::trace!(name = "ingame-behavior-on-refresh-individual-orders-trigger-spawn-order", i=?i, order=?order);
            commands.trigger(SpawnIndividualOrder(i, order.clone()));
        }
    }

    // Search for missing ones
    if let Some(orders) = orders.get(&i) {
        for (order, _) in orders {
            if orders_.iter().find(|o| o == &order).is_none() {
                tracing::trace!(name = "ingame-behavior-on-refresh-individual-orders-trigger-despawn-order", i=?i, order=?order);
                commands.trigger(DespawnIndividualOrder(i, order.clone()));
            }
        }
    }
}

fn on_refresh_squad_orders(
    event: On<RefreshSquadsOrdersEvent>,
    orders: Res<SquadOrders>,
    mut commands: Commands,
) {
    let (i, orders_) = (event.0, &event.1);
    tracing::trace!(name = "ingame-behavior-on-refresh-squad-orders", i=?i, order=?orders_);

    // Search for new ones
    for (o, order) in orders_.iter().rev().enumerate() {
        if orders
            .get(&i)
            .and_then(|orders| orders.iter().find(|(o, _)| o == order))
            .is_none()
        {
            tracing::trace!(name = "ingame-behavior-on-refresh-squad-orders-trigger-spawn-order", i=?i, order=?order);

            if let Some(spawn) = match order {
                Order::Idle => None,
                Order::MoveTo(_) | Order::MoveFastTo(_) | Order::SneakTo(_) => Some(
                    SpawnSquadOrder::Position(i, OrderIndex(o as u32), order.clone()),
                ),
                Order::Defend(_) | Order::Hide(_) => {
                    Some(SpawnSquadOrder::Direction(i, order.clone(), true))
                }
            } {
                commands.trigger(spawn);
            }
        }
    }

    // FIXME: dans les logs vu deux fois le despawn, bizarre
    // Search for missing ones
    if let Some(orders) = orders.get(&i) {
        for (order, _) in orders {
            if orders_.iter().find(|o| o == &order).is_none() {
                tracing::trace!(name = "ingame-behavior-on-refresh-squad-orders-trigger-despawn-order", i=?i, order=?order);
                commands.trigger(DespawnSquadOrder(i, order.clone()));
            }
        }
    }
}

fn on_spawn_individual_order(
    event: On<SpawnIndividualOrder>,
    g: Res<GameConfig>,
    mut orders: ResMut<IndividualOrders>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    let_some!(g = &g.0, return);
    let image = asset_server.load(UI_FILE);
    let_some!(position = event.1.position(), return);
    let rect = event.1.individual_sprite().rect();
    let x = position.x;
    let y = position.y;
    let translation = Vec3::new(x as f32, (y as f32).to_gui_y(&g.w), draw::Z_INDIV_ORDER);

    tracing::trace!(name = "ingame-behavior-on-spawn-individual-order-spawn", i=?event.0, position=?position, rect=?rect, translation=?translation);

    let sprite = Sprite {
        image,
        rect: Some(rect),
        ..default()
    };
    let transform = Transform::from_translation(translation);
    let entity = commands.spawn((sprite, transform)).id();

    orders
        .entry(event.0)
        .or_insert_with(|| vec![])
        .push((event.1.clone(), entity));
}

fn on_despawn_individual_order(
    event: On<DespawnIndividualOrder>,
    mut orders: ResMut<IndividualOrders>,
    mut commands: Commands,
) {
    tracing::trace!(name = "ingame-behavior-on-despawn-individual-order", i=?event.0, order=?event.1);
    if let Some(orders) = orders.get_mut(&event.0) {
        if let Some(x) = orders.iter().position(|(o, _)| o == &event.1) {
            let (_, entity) = orders.remove(x);
            commands.entity(entity).despawn();
        }
    }
}

fn on_despawn_individual_orders(
    event: On<DespawnIndividualOrders>,
    mut orders: ResMut<IndividualOrders>,
    mut commands: Commands,
) {
    if let Some(orders) = orders.get_mut(&event.0) {
        for (_, entity) in orders {
            commands.entity(*entity).despawn();
        }
    }
    orders.remove(&event.0);
}

/// Spawn squad orders in listened region
fn on_listening_region(region: On<ListeningRegion>, world: Res<World>, mut commands: Commands) {
    if let Some(squads) = world.squads.get(&region.0) {
        for (i, squad) in squads {
            commands.trigger(SpawnSquadOrders(*i, squad.orders.clone()))
        }
    }
}

/// Despawn squad orders in forgotten region
fn on_forgotten_region(region: On<ForgottenRegion>, world: Res<World>, mut commands: Commands) {
    if let Some(squads) = world.squads.get(&region.0) {
        for (i, _) in squads {
            commands.trigger(DespawnSquadOrders(*i))
        }
    }
}

fn on_spawn_squad_order(
    event: On<SpawnSquadOrder>,
    g: Res<GameConfig>,
    mut orders: ResMut<SquadOrders>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    world: Res<crate::world::World>,
) {
    let_some!(g = &g.0, return);
    let image = asset_server.load(UI_FILE);
    let (squad, order) = (event.squad(), event.order());
    let rect = order.sprite().rect();

    tracing::trace!(name = "ingame-behavior-on-spawn-squad-orders-spawn", i=?squad, event=?event);

    let sprite = Sprite {
        image,
        rect: Some(rect),
        ..default()
    };
    let entity = match *event {
        SpawnSquadOrder::Position(_, index, _) => {
            let marker = PositionSquadOrder(squad, index);

            let_some!(position = order.position(), return);
            let x = position.x;
            let y = position.y;
            let translation = Vec3::new(x as f32, (y as f32).to_gui_y(&g.w), draw::Z_SQUAD_ORDER);
            let transform = Transform::from_translation(translation);

            tracing::trace!(name = "ingame-behavior-on-spawn-squad-orders-spawn-position", i=?squad, order=?order);

            commands
                .spawn((
                    marker,
                    sprite,
                    transform,
                    Pickable::default(),
                    Selected::default(),
                    Dragged::<PositionSquadOrder>::default(),
                ))
                .observe(
                    drag::on_drag_start::<PositionSquadOrder>
                        .run_if(in_state(AppState::InGame))
                        .run_if(in_state(InGameState::Battle))
                        .run_if(in_state(LeftClickModeType::Select)),
                )
                .id()
        }
        SpawnSquadOrder::Direction(_, _, pending) => {
            let (marker, direction) = match order {
                Order::Idle | Order::MoveTo(_) | Order::MoveFastTo(_) | Order::SneakTo(_) => return,
                Order::Defend(direction) => (DirectionSquadOrder::Defend(squad), direction),
                Order::Hide(direction) => (DirectionSquadOrder::Hide(squad), direction),
            };
            let_some!(squad_ = world.squad(squad), return);
            let point = squad_.position;
            let rotation = direction.bquat(V::Gui);
            let transform = Transform::from_xyz(point.x, point.y.to_gui_y(&g.w), Z_SQUAD_ORDER);
            let transform = transform.with_rotation(rotation);

            tracing::trace!(name = "ingame-behavior-on-spawn-squad-orders-spawn-direction", i=?squad, order=?order);
            let mut entity = commands.spawn((
                marker,
                sprite,
                transform,
                Pickable::default(),
                Selected::default(),
                Dragged::<DirectionSquadOrder>::default(),
            ));
            if pending {
                entity.insert(PendingOrder);
            }

            entity
                .observe(
                    drag::on_drag_start::<DirectionSquadOrder>
                        .run_if(in_state(AppState::InGame))
                        .run_if(in_state(InGameState::Battle))
                        .run_if(in_state(LeftClickModeType::Select)),
                )
                .id()
        }
    };

    orders
        .entry(squad)
        .or_insert_with(|| vec![])
        .push((order.clone(), entity));
}

fn on_spawn_squad_orders(event: On<SpawnSquadOrders>, mut commands: Commands) {
    let (i, orders) = (event.0, &event.1);

    for (o, order) in orders.iter().rev().enumerate() {
        if let Some(event) = match order {
            Order::Idle => None,
            Order::MoveTo(_) | Order::MoveFastTo(_) | Order::SneakTo(_) => Some(
                SpawnSquadOrder::Position(i, OrderIndex(o as u32), order.clone()),
            ),
            Order::Defend(_) | Order::Hide(_) => {
                Some(SpawnSquadOrder::Direction(i, order.clone(), false))
            }
        } {
            commands.trigger(event);
        }
    }
}

fn on_despawn_squad_order(
    event: On<DespawnSquadOrder>,
    mut orders: ResMut<SquadOrders>,
    mut commands: Commands,
) {
    tracing::trace!(name = "ingame-behavior-on-despawn-squad-order", i=?event.0, event=?event);
    if let Some(orders) = orders.get_mut(&event.0) {
        if let Some(x) = orders.iter().position(|(o, _)| o == &event.1) {
            let (_, entity) = orders.remove(x);
            commands.entity(entity).despawn();
        }
    }
}

fn on_despawn_squad_orders(
    event: On<DespawnSquadOrders>,
    mut orders: ResMut<SquadOrders>,
    mut commands: Commands,
) {
    tracing::trace!(name = "ingame-behavior-on-despawn-squad-orders", event=?event);
    if let Some(orders) = orders.get_mut(&event.0) {
        for (_, entity) in orders {
            commands.entity(*entity).despawn();
        }
    }
    orders.remove(&event.0);
}
