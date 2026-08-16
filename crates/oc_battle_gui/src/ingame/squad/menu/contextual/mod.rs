use bevy::prelude::*;
use oc_individual::order::OrderType;
use oc_root::geo::WorldVec2;
use strum_macros::{Display, EnumIter};

use crate::{
    ingame::{
        draw::UI_FILE,
        input::left_click::{LeftClickMode, SetLeftClick},
    },
    menu::contextual::{self, ContextualMenu, open::OpenContextualMenu},
};

const ITEMS_IDLE_START_X: f32 = 0.;
const ITEMS_IDLE_START_Y: f32 = 0.;
const ITEMS_HOVER_START_X: f32 = 71.;
const ITEMS_HOVER_START_Y: f32 = 0.;
const ITEM_HEIGHT: f32 = 16.;
const ITEM_WIDTH: f32 = 70.;

#[derive(Debug, Event)]
pub struct PrepareOpenSquadContextualMenu(pub WorldVec2);

#[derive(Debug, Event)]
pub struct Open(pub WorldVec2);

#[derive(Debug)]
pub struct Menu;

#[derive(Debug, Clone, Copy, Event, Component, EnumIter, Display)]
pub enum Choice {
    Move,
    MoveFast,
    Sneak,
}

impl OpenContextualMenu<Menu> for Open {
    fn position(&self) -> WorldVec2 {
        self.0
    }
}

impl ContextualMenu for Menu {
    type OpenEvent = Open;
    type ChoiceEvent = Choice;
    type Choices = Choice;

    fn image() -> &'static str {
        UI_FILE
    }
}

impl contextual::choice::Choice<Choice> for Choice {
    fn name(&self) -> String {
        self.to_string()
    }

    fn event(&self) -> Choice {
        *self
    }

    fn idle(&self) -> Rect {
        let index = match self {
            Choice::Move => 0,
            Choice::MoveFast => 1,
            Choice::Sneak => 2,
        };

        let yp = index as f32 * ITEM_HEIGHT;
        let x0 = ITEMS_IDLE_START_X;
        let y0 = ITEMS_IDLE_START_Y + yp;
        let x1 = x0 + ITEM_WIDTH;
        let y1 = y0 + ITEM_HEIGHT;
        Rect::new(x0, y0, x1, y1)
    }

    fn hover(&self) -> Rect {
        let index = match self {
            Choice::Move => 0,
            Choice::MoveFast => 1,
            Choice::Sneak => 2,
        };

        let yp = index as f32 * ITEM_HEIGHT;
        let x0 = ITEMS_HOVER_START_X;
        let y0 = ITEMS_HOVER_START_Y + yp;
        let x1 = x0 + ITEM_WIDTH;
        let y1 = y0 + ITEM_HEIGHT;
        Rect::new(x0, y0, x1, y1)
    }
}

pub fn on_prepare_open_squad_contextual_menu(
    event: On<PrepareOpenSquadContextualMenu>,
    mut commands: Commands,
) {
    tracing::debug!("Trigger open contextual menu");
    commands.trigger(Open(event.0))
}

pub fn on_choose(item: On<Choice>, mut commands: Commands) {
    match *item {
        Choice::Move => {
            commands.trigger(SetLeftClick(LeftClickMode::Order(OrderType::MoveTo)));
        }
        Choice::MoveFast => {
            commands.trigger(SetLeftClick(LeftClickMode::Order(OrderType::MoveFastTo)));
        }
        Choice::Sneak => {
            commands.trigger(SetLeftClick(LeftClickMode::Order(OrderType::SneakTo)));
        }
    }
}
