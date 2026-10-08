use std::time::Duration;

use bevy::prelude::*;
use oc_individual::{Individual, IndividualIndex};
use oc_utils::{every, let_ok, let_some};

use crate::{
    ingame::{hud::GAP, state::SelectionUpdated},
    world::World,
};

const ROW_BG: Color = Color::srgb(0.2, 0.25, 0.2);
const ROW_HEIGHT: f32 = 20.;
const COLUMNS_WIDTH: f32 = 200.;

#[derive(Component, Default, Clone)]
pub struct SquadDetails;

#[derive(Component)]
pub struct IndividualRow;

#[derive(Component, Default, Clone)]
pub struct IndividualStatusText(pub Option<IndividualIndex>);

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

fn row(i: IndividualIndex, individual: &Individual) -> impl Scene {
    let status = individual.status.hud_label().to_string();

    bsn! {
        Node {
            width: percent(100),
            height: px(ROW_HEIGHT),
            flex_shrink: 0.,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            padding: UiRect::horizontal(px(2)),
        }
        BackgroundColor(ROW_BG)
        Children [
            (
                Text({format!("Individual {}", i.0)})
                TextFont { font_size: px(12) }
                Pickable::IGNORE
            ),
            (
                IndividualStatusText(i)
                Text(status)
                TextFont { font_size: px(12) }
                TextColor(Color::srgb(0.3, 0.5, 1.0))
                Pickable::IGNORE
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
) {
    for row in &rows {
        commands.entity(row).despawn();
    }

    let_ok!(details = details.single(), return);
    let_some!(squad = selection.selected_squads.first().copied(), return);
    let_some!(squad = world.squad(squad), return);

    for i in squad.members.iter().copied() {
        let_some!(individual = world.get_individual(i), continue);

        commands
            .spawn_scene(row(i, individual))
            .insert((IndividualRow, ChildOf(details)));
    }
}

pub fn update_rows(
    world: Res<World>,
    mut texts: Query<(&IndividualStatusText, &mut Text)>,
    time: Res<Time>,
    mut timer: Local<Option<Timer>>,
) {
    every!(timer, time, Duration::from_secs(1));

    for (individual, mut text) in &mut texts {
        let_some!(individual = individual.0, continue);
        let_some!(individual = world.get_individual(individual), continue);
        text.0 = individual.status.hud_label().to_string();
    }
}
