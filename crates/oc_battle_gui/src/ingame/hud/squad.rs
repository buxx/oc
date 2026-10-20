use std::time::Duration;

use bevy::prelude::*;
use oc_individual::{Individual, IndividualIndex, Status};
use oc_mod::{Mod, illustration::IllustrationKind};
use oc_root::files::FilesAsGui;
use oc_utils::{every, let_ok, let_some};

use crate::{
    ingame::{hud::GAP, state::SelectionUpdated},
    sprites::SpriteRect,
    states::GameConfig,
    world::World,
};

const ROW_BG: Color = Color::srgb(0.2, 0.25, 0.2);
const STATUS_OPERATIONAL_BG: Color = Color::srgb(0.1, 0.5, 0.15);
const STATUS_DEAD_BG: Color = Color::srgb(0.25, 0.25, 0.25);
const ROW_HEIGHT: f32 = 32.;
const COLUMNS_WIDTH: f32 = 200.;
const INDIVIDUAL_ICON_WIDTH: f32 = 30.;
const INDIVIDUAL_ICON_HEIGHT: f32 = 30.;

#[derive(Component, Default, Clone)]
pub struct SquadDetails;

#[derive(Component)]
pub struct IndividualRow;

#[derive(Component, Default, Clone)]
pub struct IndividualNameText(pub Option<IndividualIndex>);

#[derive(Component, Default, Clone)]
pub struct IndividualStatusText(pub Option<IndividualIndex>);

#[derive(Component, Default, Clone)]
pub struct IndividualRoundsText(pub Option<IndividualIndex>);

pub fn squad() -> impl Scene {
    bsn! {
        SquadDetails
        Node {
            width: px(COLUMNS_WIDTH),
            height: percent(100),
            flex_shrink: 0.,
            flex_direction: FlexDirection::Column,
            row_gap: px(GAP / 2.),
            overflow: Overflow::scroll_y(),
        }
    }
}

fn row(
    g: &oc_network::GameConfig,
    files: &FilesAsGui,
    i: IndividualIndex,
    individual: &Individual,
) -> impl Scene {
    let sprites = files.sprites();
    let sprite = sprites.join("illustrations.png");
    let kind = IllustrationKind::IngameIndividual;
    let illustration = g.mod_.illustration(kind, individual.illustration);
    let rect = illustration.inner().rect();
    let action = individual.action_hud_label().to_string();
    let rounds = format!("{} rnds.", primary_rounds(&g.mod_, individual));
    let status_color = status_color(individual.status);

    bsn! {
        Node {
            width: percent(100),
            height: px(ROW_HEIGHT),
            flex_shrink: 0.,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Start,
            padding: UiRect::horizontal(px(2)),
        }
        BackgroundColor(ROW_BG)
        Children [
            // Icon
            (
                Node { width: px(INDIVIDUAL_ICON_WIDTH), height: px(INDIVIDUAL_ICON_HEIGHT), flex_shrink: 0. }
                ImageNode { image: sprite, rect: {Some(rect)} }
                Pickable::IGNORE
            ),
            Node { flex_direction: FlexDirection::Column }
            Pickable::IGNORE
            Children [
                // Name
                (
                    IndividualNameText(i)
                    Text({format!("Individual {}", i.0)})
                    TextFont { font_size: px(11) }
                    BackgroundColor(status_color)
                    Pickable::IGNORE
                ),
                // Status
                (
                    IndividualStatusText(i)
                    Text(action)
                    TextFont { font_size: px(11) }
                    TextColor(Color::srgb(0.3, 0.5, 1.0))
                    Pickable::IGNORE
                ),
            ],
            //
            (
                Node { flex_direction: FlexDirection::Column }
                Pickable::IGNORE
                Children [
                    (
                        IndividualRoundsText(i)
                        Text(rounds)
                        TextFont { font_size: px(10) }
                        Pickable::IGNORE
                    ),
                ]
            ),
        ]
    }
}

pub fn spawn_rows(
    selection: On<SelectionUpdated>,
    mut commands: Commands,
    world: Res<World>,
    details: Query<Entity, With<SquadDetails>>,
    rows: Query<Entity, With<IndividualRow>>,
    g: Res<GameConfig>,
    files: Res<crate::states::GuiFiles>,
) {
    let_some!(g = &g.0, return);
    let_some!(files = &files.0, return);

    for row in &rows {
        commands.entity(row).despawn();
    }

    let_ok!(details = details.single(), return);
    let_some!(squad = selection.selected_squads.first().copied(), return);
    let_some!(squad = world.squad(squad), return);

    for i in squad.members.iter().copied() {
        let_some!(individual = world.individual(i), continue);

        commands
            .spawn_scene(row(g, files, i, individual))
            .insert((IndividualRow, ChildOf(details)));
    }
}

pub fn update_rows(
    world: Res<World>,
    g: Res<GameConfig>,
    mut texts: Query<(&IndividualStatusText, &mut Text), Without<IndividualRoundsText>>,
    mut rounds: Query<(&IndividualRoundsText, &mut Text), Without<IndividualStatusText>>,
    mut names: Query<(&IndividualNameText, &mut BackgroundColor)>,
    time: Res<Time>,
    mut timer: Local<Option<Timer>>,
) {
    let_some!(g = &g.0, return);
    every!(timer, time, Duration::from_secs(1));

    for (individual, mut text) in &mut texts {
        let_some!(individual = individual.0, continue);
        let_some!(individual = world.individual(individual), continue);
        text.0 = individual.action_hud_label().to_string();
    }

    for (individual, mut bg) in &mut names {
        let_some!(individual = individual.0, continue);
        let_some!(individual = world.individual(individual), continue);
        bg.0 = status_color(individual.status);
    }

    for (individual, mut text) in &mut rounds {
        let_some!(individual = individual.0, continue);
        let_some!(individual = world.individual(individual), continue);
        text.0 = format!("{} rnds.", primary_rounds(&g.mod_, individual));
    }
}

fn status_color(status: Status) -> Color {
    match status {
        Status::Operational => STATUS_OPERATIONAL_BG,
        Status::Dead => STATUS_DEAD_BG,
    }
}

fn primary_rounds(mod_: &Mod, individual: &Individual) -> u32 {
    let_some!(primary = &individual.weapons.primary, return 0);
    let weapon = mod_.weapon(primary.i);
    let compatibles = weapon.magazines();
    let carried: u32 = individual
        .magazines
        .iter()
        .filter_map(|carried| compatibles.iter().find(|m| m.index() == *carried))
        .map(|m| m.capacity() as u32)
        .sum();

    primary.filled_count as u32 + carried
}
