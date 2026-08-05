use bevy::prelude::*;
use oc_individual::order::OrderType;
use oc_physics::update::bevy::Position;
use oc_root::{
    WcfgFrom,
    geo::{ScreenVec2, ScreenVec3},
};
use oc_utils::{bevy::EntityMapping, let_ok, let_some};

use crate::{
    entity::individual::IndividualIndex,
    ingame::input::left_click::{LeftClickMode, SetLeftClick},
    menu::contextual::{Content, ContextMenuItem, ContextualMenu, open::OpenContextualMenuEvent},
    states::GameConfig,
};

#[derive(Debug, Event)]
pub struct PrepareOpenSquadContextualMenu(pub oc_individual::IndividualIndex);

#[derive(Debug, Event)]
pub struct Open(pub ScreenVec2, pub Content<Choice>);

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
    fn position(&self) -> ScreenVec2 {
        self.0
    }

    fn content(&self) -> &Content<Choice> {
        &self.1
    }
}

pub fn on_prepare_open_squad_contextual_menu(
    event: On<PrepareOpenSquadContextualMenu>,
    individuals: Res<EntityMapping<oc_individual::IndividualIndex>>,
    g: Res<GameConfig>,
    mut commands: Commands,
    camera_query: Single<(&Camera, &GlobalTransform)>,
    query: Query<(&IndividualIndex, &Position)>,
) {
    let_some!(g = &g.0, return);
    let (camera, camera_transform) = *camera_query;
    let_some!(individual = individuals.get(&event.0), return);
    let_ok!((_i, position) = query.get(*individual), return);
    let position = position.0;
    let position = ScreenVec3::from_(position, &g.w);
    let position = camera.world_to_viewport(camera_transform, position.into());
    let_ok!(position = position, return);
    let position = ScreenVec2::new(position.x, position.y);

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
