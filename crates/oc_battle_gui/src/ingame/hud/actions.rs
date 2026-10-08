use bevy::prelude::*;

use crate::{
    ingame::BattlePhase,
    window::{self, ToggleWindow},
};

const BUTTON_BG: Color = Color::srgb(0.25, 0.25, 0.25);
const BUTTON_BG_HOVER: Color = Color::srgb(0.4, 0.4, 0.4);

#[derive(Component, Default, Clone)]
pub struct BattlePhaseButton;

#[derive(Component, Default, Clone)]
pub struct ActionButtonText;

pub fn actions() -> impl Scene {
    bsn! {
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: px(2),
            height: percent(100),
            flex_shrink: 0.,
        }
        Children [
            (
                BattlePhaseButton
                battle_state_button()
                on(on_battle_phase_click)
            ),
        ]
    }
}

fn battle_state_button() -> impl Scene {
    bsn! {
        Node {
            padding: UiRect::axes(px(12), px(6)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
        }
        BackgroundColor(BUTTON_BG)
        on(on_hover)
        on(on_out)
        Children [
            (
                Text("n/a")
                TextFont { font_size: px(12) }
                ActionButtonText Pickable::IGNORE
            )
        ]
    }
}

fn on_hover(e: On<Pointer<Over>>, mut q: Query<&mut BackgroundColor>) {
    if let Ok(mut bg) = q.get_mut(e.entity) {
        bg.0 = BUTTON_BG_HOVER;
    }
}

fn on_out(e: On<Pointer<Out>>, mut q: Query<&mut BackgroundColor>) {
    if let Ok(mut bg) = q.get_mut(e.entity) {
        bg.0 = BUTTON_BG;
    }
}

fn on_battle_phase_click(
    _: On<Pointer<Click>>,
    mut commands: Commands,
    phase: Res<State<BattlePhase>>,
    fight_menu: Res<window::battle::fight::FightWindow>,
) {
    match phase.get() {
        BattlePhase::Deployment => {
            let window = fight_menu
                .0
                .clone()
                .unwrap_or(window::battle::fight::Window);
            commands.trigger(ToggleWindow(window::Window::Fight(window)));
        }
        BattlePhase::Fight => {
            // TODO: surrender
            tracing::debug!("Surrender");
        }
    }
}

pub fn update_fight_button(
    phase: Res<State<BattlePhase>>,
    mut texts: Query<(&mut Text, &ChildOf), With<ActionButtonText>>,
    buttons: Query<(), With<BattlePhaseButton>>,
) {
    for (mut text, parent) in &mut texts {
        if !buttons.contains(parent.parent()) {
            continue;
        }
        let label = match phase.get() {
            BattlePhase::Deployment => "Fight",
            BattlePhase::Fight => "Surrender",
        };
        if text.0 != label {
            text.0 = label.to_string();
        }
    }
}
