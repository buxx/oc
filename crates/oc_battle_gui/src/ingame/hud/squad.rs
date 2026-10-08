use std::time::Duration;

use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::*,
};
use oc_individual::{
    order::Order,
    squad::{Squad, SquadIndex},
};
use oc_mod::illustration::IllustrationKind;
use oc_root::files::Files;
use oc_utils::{let_ok, let_some};

use crate::{
    config,
    ingame::{camera::GoToPoint, hud::SCROLL_LINE_PX},
    network,
    sprites::SpriteRect,
    states::GameConfig,
};
use crate::{ingame::hud::GAP, world::World};

const CELL_BG: Color = Color::srgb(0.2, 0.25, 0.2);
const CELL_BG_HOVER: Color = Color::srgb(0.35, 0.45, 0.35);

const SQUAD_CELL_WIDTH: f32 = 122.;
const SQUAD_CELL_HEIGHT: f32 = 52.;

#[derive(Component, Default, Clone)]
pub struct SquadGrid;

#[derive(Component, Default, Clone)]
pub struct SquadBehaviorText;

#[derive(Component)]
pub struct SquadCell(pub SquadIndex);

pub fn grid() -> impl Scene {
    bsn! {
        SquadGrid
        Node {
            flex_grow: 1.,
            height: percent(100),
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            align_content: AlignContent::FlexStart,
            row_gap: px(GAP),
            column_gap: px(GAP),
            overflow: Overflow::scroll_y(),
        }
    }
}

fn squad_cell(
    g: &oc_network::GameConfig,
    connect: &config::Connect,
    i: SquadIndex,
    squad: &Squad,
) -> impl Scene {
    let mod__ = g.mod_.canonical();
    let world = g.meta.canonical();
    let files = Files::new(mod__, world).into_gui(g.static_.clone(), connect.clone().into());
    let sprites = files.sprites();
    let sprite = sprites.join("illustrations.png");
    let kind = IllustrationKind::IngameSquad;
    let illustration = g.mod_.illustration(kind, squad.illustration);
    let rect = illustration.inner().rect();

    bsn! {
        Node {
            width: px(SQUAD_CELL_WIDTH),
            height: px(SQUAD_CELL_HEIGHT),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            column_gap: px(4),
            padding: UiRect::all(px(4)),
        }
        BackgroundColor(CELL_BG)
        on(on_hover)
        on(on_out)
        on(on_click)
        Children [
            // Squad icon
            (
                Node { width: px(50), height: px(50), flex_shrink: 0. }
                ImageNode { image: sprite, rect: {Some(rect)} }
                Pickable::IGNORE
            ),
            // Squad name & behavior
            (
                Node { flex_direction: FlexDirection::Column }
                Pickable::IGNORE
                Children [
                    (
                        Text({format!("Squad {}", i.0)})
                        TextFont { font_size: px(14) }
                        Pickable::IGNORE
                        BackgroundColor(Color::srgb(0.1, 0.5, 0.1))
                    ),
                    (
                        SquadBehaviorText
                        Text("Behavior")
                        TextFont { font_size: px(14) }
                        Pickable::IGNORE
                        TextColor(Color::srgb(0.3, 0.5, 1.0))
                    ),
                ]
            ),
        ]
    }
}

pub fn spawn_squad_cells(
    mut commands: Commands,
    world: Res<World>,
    grid: Query<Entity, With<SquadGrid>>,
    cells: Query<(), With<SquadCell>>,
    g: Res<GameConfig>,
    network: Res<network::state::State>,
) {
    let_some!(g = &g.0, return);
    let_some!(connect = &network.server, return);
    // Wait grid exist
    let Ok(grid) = grid.single() else { return };
    // Wait squads received and do only if cells not already spawned
    if !cells.is_empty() || world.squads_refs.is_empty() {
        return;
    }

    for i in world.squads() {
        let_some!(squad = world.squad(i), continue);
        commands
            .spawn_scene(squad_cell(g, &connect, i, squad))
            .insert((SquadCell(i), ChildOf(grid)));
    }
}

fn on_hover(e: On<Pointer<Over>>, mut q: Query<&mut BackgroundColor>) {
    if let Ok(mut bg) = q.get_mut(e.entity) {
        bg.0 = CELL_BG_HOVER;
    }
}

fn on_out(e: On<Pointer<Out>>, mut q: Query<&mut BackgroundColor>) {
    if let Ok(mut bg) = q.get_mut(e.entity) {
        bg.0 = CELL_BG;
    }
}

fn on_click(
    e: On<Pointer<Click>>,
    mut commands: Commands,
    cells: Query<&SquadCell>,
    world: Res<World>,
) {
    let_ok!(cell = cells.get(e.entity), return);
    let_some!(squad = world.squad(cell.0), return);
    commands.trigger(GoToPoint(squad.position));
}

pub fn update_squad_behaviors(
    world: Res<World>,
    mut texts: Query<(Entity, &mut Text), With<SquadBehaviorText>>,
    cells: Query<&SquadCell>,
    parents: Query<&ChildOf>,
    time: Res<Time>,
    mut timer: Local<Option<Timer>>,
) {
    static PERIOD: Duration = Duration::from_secs(1);
    let timer = timer.get_or_insert_with(|| Timer::new(PERIOD, TimerMode::Repeating));
    if !timer.tick(time.delta()).just_finished() {
        return;
    }

    for (entity, mut text) in &mut texts {
        let cell = parents
            .iter_ancestors(entity)
            .find_map(|e| cells.get(e).ok());
        let_some!(cell = cell, continue);
        let_some!(squad = world.squad(cell.0), continue);

        let label = match squad.orders.first() {
            None => "No order",
            Some(Order::Idle) => "Idle",
            Some(Order::MoveTo(_)) => "Move",
            Some(Order::MoveFastTo(_)) => "Move fast",
            Some(Order::SneakTo(_)) => "Sneak",
            Some(Order::Defend(_)) => "Defend",
            Some(Order::Hide(_)) => "Hide",
            Some(Order::Engage(_)) => "Engage",
            Some(Order::Suppress(_)) => "Suppress",
        };
        if text.0 != label {
            text.0 = label.to_string();
        }
    }
}

pub fn scroll_squad_grid(
    mut wheel: MessageReader<MouseWheel>,
    mut grid: Query<(&mut ScrollPosition, &ComputedNode, &UiGlobalTransform), With<SquadGrid>>,
    window: Single<&Window>,
) {
    let_some!(cursor = window.physical_cursor_position(), return);
    let Ok((mut scroll, computed, transform)) = grid.single_mut() else {
        return;
    };
    if !computed.contains_point(*transform, cursor) {
        return;
    }

    for event in wheel.read() {
        let dy = match event.unit {
            MouseScrollUnit::Line => event.y * SCROLL_LINE_PX,
            MouseScrollUnit::Pixel => event.y,
        };
        let max = (computed.content_size().y - computed.size().y).max(0.)
            * computed.inverse_scale_factor();
        scroll.y = (scroll.y - dy).clamp(0., max);
    }
}
