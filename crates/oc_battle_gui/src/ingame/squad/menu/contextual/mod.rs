use bevy::prelude::*;
use oc_individual::order::OrderType;
use oc_root::geo::WorldVec2;

use crate::{
    ingame::input::left_click::{LeftClickMode, SetLeftClick},
    menu::contextual::{Content, ContextMenuItem, ContextualMenu, open::OpenContextualMenuEvent},
};

#[derive(Debug, Event)]
pub struct PrepareOpenSquadContextualMenu(pub WorldVec2);

#[derive(Debug, Event)]
pub struct Open(pub WorldVec2, pub Content<Choice>);

pub struct Menu;

#[derive(Debug, Clone, Event)]
pub enum Choice {
    Move,
    MoveFast,
}

impl ContextualMenu for Menu {
    type OpenEvent = Open;
    type ChoiceEvent = Choice;
}

impl OpenContextualMenuEvent<Choice> for Open {
    fn position(&self) -> WorldVec2 {
        self.0
    }

    fn content(&self) -> &Content<Choice> {
        &self.1
    }
}

pub fn on_prepare_open_squad_contextual_menu(
    event: On<PrepareOpenSquadContextualMenu>,
    mut commands: Commands,
) {
    let position = event.0;
    // FIXME: Menu choices according to selected squad
    let items = vec![
        ContextMenuItem::new("move".to_string(), Choice::Move),
        ContextMenuItem::new("move fast".to_string(), Choice::MoveFast),
    ];
    let content = crate::menu::contextual::Content::new(items);
    let open = Open(position, content);
    tracing::debug!("Trigger open contextual menu");
    commands.trigger(open)
}

pub fn on_choose(item: On<Choice>, mut commands: Commands) {
    match *item {
        Choice::Move => {
            commands.trigger(SetLeftClick(LeftClickMode::Order(OrderType::MoveTo)));
        }
        Choice::MoveFast => {
            commands.trigger(SetLeftClick(LeftClickMode::Order(OrderType::MoveFastTo)));
        }
    }
}
