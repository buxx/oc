use bevy::prelude::*;
use oc_individual::order::Order;
use oc_physics::update::bevy::Position;
use oc_root::y::Y;
use oc_utils::d2;
use rustc_hash::FxHashMap;

use crate::{
    entity::individual::{IndividualIndex, Intent},
    ingame::{draw, individual::SetOrdersEvent},
    states::{GameConfig, InGameState},
};

const PATH_COLOR: Color = Color::srgba(1.0, 1.0, 1.0, 0.15);

pub struct BehaviorPlugin;

#[derive(Default, Reflect, GizmoConfigGroup)]
struct PathGizmos;

#[derive(Debug, Resource, Default, Deref, DerefMut)]
pub struct Orders(FxHashMap<oc_individual::IndividualIndex, Vec<(Order, Entity)>>);

#[derive(Debug, Event)]
pub struct RefreshOrdersEvent(
    pub oc_individual::IndividualIndex,
    pub Vec<oc_individual::order::Order>,
);

#[derive(Debug, Event)]
pub struct SpawnOrder(oc_individual::IndividualIndex, oc_individual::order::Order);

#[derive(Debug, Event)]
pub struct DespawnOrder(
    pub oc_individual::IndividualIndex,
    pub oc_individual::order::Order,
);

#[derive(Debug, Event)]
pub struct DespawnOrders(pub oc_individual::IndividualIndex);

pub enum OrderSprite {
    Move,
}

impl OrderSprite {
    pub fn rect(&self) -> Rect {
        match self {
            OrderSprite::Move => Rect::new(0., 100., 11., 111.),
        }
    }
}

impl Plugin for BehaviorPlugin {
    fn build(&self, app: &mut App) {
        app.init_gizmo_group::<PathGizmos>()
            .init_resource::<Orders>()
            .add_observer(on_set_orders_event)
            .add_observer(on_refresh_orders_event)
            .add_observer(on_spawn_order)
            .add_observer(on_despawn_order)
            .add_observer(on_despawn_orders)
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

fn on_set_orders_event(event: On<SetOrdersEvent>, mut commands: Commands) {
    tracing::trace!(name = "ingame-behavio-on-set-orders-trigger-refresh-orders", event=?event);
    commands.trigger(RefreshOrdersEvent(event.0, event.1.clone()))
}

fn on_refresh_orders_event(
    event: On<RefreshOrdersEvent>,
    orders: Res<Orders>,
    mut commands: Commands,
) {
    let (i, orders_) = (event.0, &event.1);

    // Search for new ones
    for order in orders_ {
        if orders
            .get(&i)
            .map(|orders| orders.iter().find(|(o, _)| o.equal(order)))
            .is_none()
        {
            tracing::trace!(name = "ingame-behavio-on-refresh-orders-trigger-spawn-orders", i=?i, order=?order);
            commands.trigger(SpawnOrder(i, order.clone()));
        }
    }

    // Search for missing ones
    if let Some(orders) = orders.get(&i) {
        for (order, _) in orders {
            if orders_.iter().find(|o| o.equal(order)).is_none() {
                tracing::trace!(name = "ingame-behavio-on-refresh-orders-trigger-despawn-orders", i=?i, order=?order);
                commands.trigger(DespawnOrder(i, order.clone()));
            }
        }
    }
}

fn on_spawn_order(
    event: On<SpawnOrder>,
    g: Res<GameConfig>,
    mut orders: ResMut<Orders>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    let Some(g) = &g.0 else {
        return;
    };
    let image = asset_server.load("ui/ui.png");
    let (rect, position) = match &event.1 {
        Order::Idle => (Some(Rect::new(0., 0., 0., 0.)), d2::Position::new(0., 0.)), // Should not happen
        Order::MoveTo(position) => (Some(OrderSprite::Move.rect()), position.clone()),
    };
    let x = position.x;
    let y = position.y;
    let translation = Vec3::new(x as f32, (y as f32).to_gui_y(&g.w), draw::Z_ORDER);

    tracing::trace!(name = "ingame-behavio-on-spawn-orders-spawn", position=?position, rect=?rect, translation=?translation);

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

fn on_despawn_order(event: On<DespawnOrder>, mut orders: ResMut<Orders>, mut commands: Commands) {
    if let Some(orders) = orders.get_mut(&event.0) {
        if let Some(x) = orders.iter().position(|(o, _)| o.equal(&event.1)) {
            let (_, entity) = orders.remove(x);
            commands.entity(entity).despawn();
        }
    }
}

fn on_despawn_orders(event: On<DespawnOrders>, mut orders: ResMut<Orders>, mut commands: Commands) {
    if let Some(orders) = orders.get_mut(&event.0) {
        for (_, entity) in orders {
            commands.entity(*entity).despawn();
        }
    }
    orders.remove(&event.0);
}
