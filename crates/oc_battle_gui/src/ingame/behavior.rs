use bevy::prelude::*;
use oc_individual::{
    order::{Order, OrderIndex},
    squad::SquadIndex,
};
use oc_network::{SquadMessage, ToServer};
use oc_root::{geo::WorldVec2, y::Y};
use oc_utils::{let_ok, let_some};
use rustc_hash::FxHashMap;

use crate::{
    ingame::{
        InGameState, draw,
        input::left_click::LeftClickModeType,
        region::{ForgottenRegion, ListeningRegion},
    },
    network::output::ToServerEvent,
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
pub struct SpawnSquadOrder(
    oc_individual::squad::SquadIndex,
    oc_individual::order::OrderIndex,
    oc_individual::order::Order,
);

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

pub enum IndividualOrderSprite {
    Move,
}

impl IndividualOrderSprite {
    pub fn rect(&self) -> Rect {
        const START_X: f32 = 22.;
        const START_Y: f32 = 100.;
        const WIDTH: f32 = 11.;
        const HEIGHT: f32 = 11.;

        let i = match self {
            Self::Move => 0,
        } as f32;

        let start_y = START_Y + (i * HEIGHT);
        Rect::new(START_X, start_y, START_X + WIDTH, start_y + HEIGHT)
    }
}

#[derive(Debug, Component)]
pub struct SquadOrder(SquadIndex, OrderIndex);

impl Dragging for SquadOrder {
    fn spawn(commands: &mut Commands, marker: Phantom) {
        commands.trigger(SpawnSquadOrderMarkerPhantom(marker));
    }

    fn drop(commands: &mut Commands, subject: Entity, point: WorldVec2) {
        commands.trigger(DropSquadOrderMarkerPhantom(subject, point));
    }
}

#[derive(Debug, Event)]
pub struct SpawnSquadOrderMarkerPhantom(Phantom);

#[derive(Debug, Event)]
pub struct DropSquadOrderMarkerPhantom(Entity, WorldVec2);

pub enum SquadOrderSprite {
    Move,
}

pub fn on_spawn_squad_order_marker_phantom(
    event: On<SpawnSquadOrderMarkerPhantom>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // FIXME BS NOW: spawn correct sprite according to order
    let image = asset_server.load("ui/ui.png");
    let rect = Some(SquadOrderSprite::Move.rect());
    let marker = event.0;
    let sprite = Sprite {
        image,
        rect,
        ..default()
    };
    commands.spawn((sprite, Transform::default(), marker));
}

pub fn on_drop_squad_order_marker_phantom(
    event: On<DropSquadOrderMarkerPhantom>,
    mut commands: Commands,
    query: Query<&mut SquadOrder>,
) {
    let_ok!(order = query.get(event.0), return);
    let (squad, index, position) = (order.0, order.1, event.1);
    let message = SquadMessage::SetOrderPosition(index, position);
    tracing::trace!(name = "ingame-behavior-on-drop-squad-order-marker-phantom", squad=?squad, index=?index, position=?position);
    commands.trigger(ToServerEvent(ToServer::Squad(squad, message)))
}

impl SquadOrderSprite {
    pub fn rect(&self) -> Rect {
        const START_X: f32 = 0.;
        const START_Y: f32 = 100.;
        const WIDTH: f32 = 11.;
        const HEIGHT: f32 = 11.;

        let i = match self {
            Self::Move => 0,
        } as f32;

        let start_y = START_Y + (i * HEIGHT);
        Rect::new(START_X, start_y, START_X + WIDTH, start_y + HEIGHT)
    }
}

impl Plugin for BehaviorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DragPlugin::<SquadOrder>::default())
            .init_resource::<IndividualOrders>()
            .init_resource::<SquadOrders>()
            // .add_observer(on_set_individual_orders)
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
            .add_observer(on_drop_squad_order_marker_phantom);
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
            .and_then(|orders| orders.iter().find(|(o, _)| o.equal(order)))
            .is_none()
        {
            tracing::trace!(name = "ingame-behavior-on-refresh-individual-orders-trigger-spawn-order", i=?i, order=?order);
            commands.trigger(SpawnIndividualOrder(i, order.clone()));
        }
    }

    // Search for missing ones
    if let Some(orders) = orders.get(&i) {
        for (order, _) in orders {
            if orders_.iter().find(|o| o.equal(order)).is_none() {
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
            .and_then(|orders| orders.iter().find(|(o, _)| o.equal(order)))
            .is_none()
        {
            tracing::trace!(name = "ingame-behavior-on-refresh-squad-orders-trigger-spawn-order", i=?i, order=?order);
            commands.trigger(SpawnSquadOrder(i, OrderIndex(o as u32), order.clone()));
        }
    }

    // FIXME BS NOW: dans les logs vu deux fois le despawn, bizarre
    // Search for missing ones
    if let Some(orders) = orders.get(&i) {
        for (order, _) in orders {
            if orders_.iter().find(|o| o.equal(order)).is_none() {
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
    let image = asset_server.load("ui/ui.png");
    let (rect, position) = match &event.1 {
        Order::Idle => (Some(Rect::new(0., 0., 0., 0.)), WorldVec2::new(0., 0.)), // Should not happen
        Order::MoveTo(position) => (Some(IndividualOrderSprite::Move.rect()), position.clone()),
    };
    let x = position.x;
    let y = position.y;
    let translation = Vec3::new(x as f32, (y as f32).to_gui_y(&g.w), draw::Z_INDIV_ORDER);

    tracing::trace!(name = "ingame-behavior-on-spawn-individual-order-spawn", i=?event.0, position=?position, rect=?rect, translation=?translation);

    let sprite = Sprite {
        image,
        rect,
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
        if let Some(x) = orders.iter().position(|(o, _)| o.equal(&event.1)) {
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
) {
    let_some!(g = &g.0, return);
    let image = asset_server.load("ui/ui.png");
    let SpawnSquadOrder(squad, index, order) = &*event;
    let (rect, position) = match &order {
        Order::Idle => (Some(Rect::new(0., 0., 0., 0.)), WorldVec2::new(0., 0.)), // Should not happen
        Order::MoveTo(position) => (Some(SquadOrderSprite::Move.rect()), position.clone()),
    };
    let x = position.x;
    let y = position.y;
    let translation = Vec3::new(x as f32, (y as f32).to_gui_y(&g.w), draw::Z_SQUAD_ORDER);

    tracing::trace!(name = "ingame-behavior-on-spawn-squad-orders-spawn", i=?squad, position=?position, rect=?rect, translation=?translation);

    let sprite = Sprite {
        image,
        rect,
        ..default()
    };
    let transform = Transform::from_translation(translation);
    let entity = commands
        .spawn((
            SquadOrder(*squad, *index),
            sprite,
            transform,
            Pickable::default(),
            Selected::default(),
            Dragged::<SquadOrder>::default(),
        ))
        .observe(
            drag::on_drag_start::<SquadOrder>
                .run_if(in_state(AppState::InGame))
                .run_if(in_state(InGameState::Battle))
                .run_if(in_state(LeftClickModeType::Select)),
        )
        .id();

    orders
        .entry(event.0)
        .or_insert_with(|| vec![])
        .push((order.clone(), entity));
}

fn on_spawn_squad_orders(event: On<SpawnSquadOrders>, mut commands: Commands) {
    let (i, orders) = (event.0, &event.1);

    for (o, order) in orders.iter().rev().enumerate() {
        commands.trigger(SpawnSquadOrder(i, OrderIndex(o as u32), order.clone()));
    }
}

fn on_despawn_squad_order(
    event: On<DespawnSquadOrder>,
    mut orders: ResMut<SquadOrders>,
    mut commands: Commands,
) {
    tracing::trace!(name = "ingame-behavior-on-despawn-squad-order", i=?event.0, event=?event);
    if let Some(orders) = orders.get_mut(&event.0) {
        if let Some(x) = orders.iter().position(|(o, _)| o.equal(&event.1)) {
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
