use bevy::prelude::*;
use oc_individual::order::Order;
use oc_physics::update::bevy::Position;
use oc_root::y::Y;
use oc_utils::d2;
use rustc_hash::FxHashMap;

use crate::{
    entity::individual::{IndividualIndex, Intent},
    ingame::{
        draw,
        individual::SetOrdersEvent,
        region::{ForgottenRegion, ListeningRegion},
    },
    states::{GameConfig, InGameState},
    world::World,
};

const PATH_COLOR: Color = Color::srgba(1.0, 1.0, 1.0, 0.15);

pub struct BehaviorPlugin;

#[derive(Default, Reflect, GizmoConfigGroup)]
struct PathGizmos;

#[derive(Debug, Resource, Default, Deref, DerefMut)]
pub struct IndividualOrders(FxHashMap<oc_individual::IndividualIndex, Vec<(Order, Entity)>>);

#[derive(Debug, Resource, Default, Deref, DerefMut)]
pub struct SquadOrders(FxHashMap<oc_individual::squad::SquadIndex, Vec<(Order, Entity)>>);

#[derive(Debug, Event)]
pub struct RefreshIndividualOrdersEvent(
    pub oc_individual::IndividualIndex,
    pub Vec<oc_individual::order::Order>,
);

#[derive(Debug, Event)]
pub struct RefreshSquadsOrdersEvent(
    pub oc_individual::squad::SquadIndex,
    pub Vec<oc_individual::order::Order>,
);

#[derive(Debug, Event)]
pub struct SpawnIndividualOrder(oc_individual::IndividualIndex, oc_individual::order::Order);

#[derive(Debug, Event)]
pub struct SpawnSquadOrder(
    oc_individual::squad::SquadIndex,
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

pub enum SquadOrderSprite {
    Move,
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
        app.init_gizmo_group::<PathGizmos>()
            .init_resource::<IndividualOrders>()
            .init_resource::<SquadOrders>()
            .add_observer(on_set_individual_orders)
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
            .add_systems(Startup, setup)
            .add_systems(Update, draw_paths.run_if(in_state(InGameState::Battle)));
    }
}

fn setup(mut config: ResMut<GizmoConfigStore>) {
    tracing::trace!(name = "ingame-behavio-setup-gizmos");
    let (gizmos, _) = config.config_mut::<PathGizmos>();
    gizmos.line.width = 1.0;
    gizmos.line.style = GizmoLineStyle::Dotted;
}

fn draw_paths(
    g: Res<GameConfig>,
    intents: Query<(&Intent, &Position), With<IndividualIndex>>,
    mut gizmos: Gizmos<PathGizmos>,
) {
    let Some(g) = &g.0 else {
        return;
    };

    for (intent, position) in intents {
        match &intent.0 {
            oc_individual::behavior::Intent::Idle(_) => {}
            oc_individual::behavior::Intent::MoveTo(_, path) => {
                let mut previous: [f32; 2] = [position.0[0], position.0[1]];
                for point in path.iter() {
                    let start = Vec3::new(previous[0], previous[1].to_gui_y(&g.w), draw::Z_PATH);
                    let stop = Vec3::new(point[0], point[1].to_gui_y(&g.w), draw::Z_PATH);
                    gizmos.line(start, stop, PATH_COLOR);

                    previous = [point[0], point[1]];
                }
            }
        }
    }
}

// FIXME BS NOW: on voit la trace toute les secondes ! (manque if != qqpart)
fn on_set_individual_orders(event: On<SetOrdersEvent>, mut commands: Commands) {
    tracing::trace!(name = "ingame-behavio-on-set-individual-orders-trigger-refresh-orders", event=?event);
    commands.trigger(RefreshIndividualOrdersEvent(event.0, event.1.clone()))
}

fn on_refresh_individual_orders(
    event: On<RefreshIndividualOrdersEvent>,
    orders: Res<IndividualOrders>,
    mut commands: Commands,
) {
    tracing::trace!(name = "ingame-behavio-on-refresh-individual-orders", event=?event);
    let (i, orders_) = (event.0, &event.1);

    // Search for new ones
    for order in orders_ {
        if orders
            .get(&i)
            .map(|orders| orders.iter().find(|(o, _)| o.equal(order)))
            .is_none()
        {
            tracing::trace!(name = "ingame-behavio-on-refresh-individual-orders-trigger-spawn-order", i=?i, order=?order);
            commands.trigger(SpawnIndividualOrder(i, order.clone()));
        }
    }

    // Search for missing ones
    if let Some(orders) = orders.get(&i) {
        for (order, _) in orders {
            if orders_.iter().find(|o| o.equal(order)).is_none() {
                tracing::trace!(name = "ingame-behavio-on-refresh-individual-orders-trigger-despawn-order", i=?i, order=?order);
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
    tracing::trace!(name = "ingame-behavio-on-refresh-squad-orders", event=?event);
    let (i, orders_) = (event.0, &event.1);

    // Search for new ones
    for order in orders_ {
        if orders
            .get(&i)
            .map(|orders| orders.iter().find(|(o, _)| o.equal(order)))
            .is_none()
        {
            tracing::trace!(name = "ingame-behavio-on-refresh-squad-orders-trigger-spawn-order", i=?i, order=?order);
            commands.trigger(SpawnSquadOrder(i, order.clone()));
        }
    }

    // Search for missing ones
    if let Some(orders) = orders.get(&i) {
        for (order, _) in orders {
            if orders_.iter().find(|o| o.equal(order)).is_none() {
                tracing::trace!(name = "ingame-behavio-on-refresh-squad-orders-trigger-despawn-order", i=?i, order=?order);
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
    let Some(g) = &g.0 else {
        return;
    };
    let image = asset_server.load("ui/ui.png");
    let (rect, position) = match &event.1 {
        Order::Idle => (Some(Rect::new(0., 0., 0., 0.)), d2::Position::new(0., 0.)), // Should not happen
        Order::MoveTo(position) => (Some(IndividualOrderSprite::Move.rect()), position.clone()),
    };
    let x = position.x;
    let y = position.y;
    let translation = Vec3::new(x as f32, (y as f32).to_gui_y(&g.w), draw::Z_INDIV_ORDER);

    tracing::trace!(name = "ingame-behavior-on-spawn-individual-orders-spawn", position=?position, rect=?rect, translation=?translation);

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
    let Some(g) = &g.0 else {
        return;
    };
    let image = asset_server.load("ui/ui.png");
    let (rect, position) = match &event.1 {
        Order::Idle => (Some(Rect::new(0., 0., 0., 0.)), d2::Position::new(0., 0.)), // Should not happen
        Order::MoveTo(position) => (Some(SquadOrderSprite::Move.rect()), position.clone()),
    };
    let x = position.x;
    let y = position.y;
    let translation = Vec3::new(x as f32, (y as f32).to_gui_y(&g.w), draw::Z_SQUAD_ORDER);

    tracing::trace!(name = "ingame-behavior-on-spawn-squad-orders-spawn", position=?position, rect=?rect, translation=?translation);

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

fn on_spawn_squad_orders(event: On<SpawnSquadOrders>, mut commands: Commands) {
    let (i, orders) = (event.0, &event.1);

    for order in orders {
        commands.trigger(SpawnSquadOrder(i, order.clone()));
    }
}

fn on_despawn_squad_order(
    event: On<DespawnSquadOrder>,
    mut orders: ResMut<SquadOrders>,
    mut commands: Commands,
) {
    tracing::trace!(name = "ingame-behavio-on-despawn-squad-order", event=?event);
    if let Some(orders) = orders.get_mut(&event.0) {
        if let Some(x) = orders.iter().position(|(o, _)| o.equal(&event.1)) {
            let (_, entity) = orders.remove(x);
            commands.entity(entity).despawn();
        }
    }
}

// FIXME BS NOW
fn on_despawn_squad_orders(
    event: On<DespawnSquadOrders>,
    mut orders: ResMut<SquadOrders>,
    mut commands: Commands,
) {
    tracing::trace!(name = "ingame-behavio-on-despawn-squad-orders", event=?event);
    if let Some(orders) = orders.get_mut(&event.0) {
        for (_, entity) in orders {
            commands.entity(*entity).despawn();
        }
    }
    orders.remove(&event.0);
}
