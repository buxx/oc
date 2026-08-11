use bevy::prelude::*;
use oc_individual::order::OrderType;
use oc_physics::update::bevy::Position;
use oc_root::{
    WcfgFrom,
    geo::{ScreenVec2, WorldVec2},
};
use oc_utils::let_some;

use crate::{
    ingame::input::left_click::{LeftClickMode, SetLeftClick},
    menu::contextual::{Content, ContextMenuItem, ContextualMenu, open::OpenContextualMenuEvent},
    states::GameConfig,
};

#[derive(Debug, Event)]
pub struct PrepareOpenSquadContextualMenu(pub WorldVec2);

#[derive(Debug, Event)]
pub struct Open(pub WorldVec2, pub Content<Choice>);

pub struct Menu;

#[derive(Debug, Clone, Event)]
pub enum Choice {
    Move,
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
    let items = vec![ContextMenuItem::new("move".to_string(), Choice::Move)];
    let content = crate::menu::contextual::Content::new(items);
    let open = Open(position, content);
    commands.trigger(open)
}

pub fn on_choose(item: On<Choice>, mut commands: Commands) {
    match *item {
        Choice::Move => commands.trigger(SetLeftClick(LeftClickMode::Order(OrderType::MoveTo))),
    }
}
