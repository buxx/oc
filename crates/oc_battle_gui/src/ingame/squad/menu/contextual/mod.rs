use bevy::prelude::*;
use oc_physics::update::bevy::Position;
use oc_root::y::Y;
use oc_utils::{let_ok, let_some};

use crate::{
    entity::individual::IndividualIndex,
    menu::contextual::{Content, ContextMenuItem, ContextualMenu, open::OpenContextualMenuEvent},
    states::GameConfig,
};

#[derive(Debug, Event)]
pub struct Open(pub Vec2, pub Content<Choice>);

pub struct Menu;

#[derive(Debug, Clone, Event)]
pub enum Choice {
    SetColor(Srgba),
}

impl ContextualMenu for Menu {
    type OpenEvent = Open;
    type ChoiceEvent = Choice;
}

impl OpenContextualMenuEvent<Choice> for Open {
    fn position(&self) -> Vec2 {
        self.0
    }

    fn content(&self) -> &Content<Choice> {
        &self.1
    }
}

pub fn on_click(
    event: On<Pointer<Click>>,
    g: Res<GameConfig>,
    mut commands: Commands,
    camera_query: Single<(&Camera, &GlobalTransform)>,
    query: Query<(&IndividualIndex, &Position)>,
) {
    let_some!(g = &g.0, return);

    if event.button == PointerButton::Secondary {
        let (camera, camera_transform) = *camera_query;
        let individual = event.original_event_target();
        let_ok!((_i, position) = query.get(individual), return);
        let position = Vec3::new(position.0[0], position.0[1], 0.0);
        let position = position.to_gui_y(&g.w);
        let position = camera.world_to_viewport(camera_transform, position);
        let_ok!(position = position, return);

        // FIXME BS NOW
        let items = vec![
            ContextMenuItem::new(
                "fuchsia".to_string(),
                Choice::SetColor(bevy::color::palettes::css::FUCHSIA),
            ),
            ContextMenuItem::new(
                "gray".to_string(),
                Choice::SetColor(bevy::color::palettes::css::GRAY),
            ),
        ];
        let content = crate::menu::contextual::Content::new(items);
        let open = Open(position, content);
        commands.trigger(open)
    }
}

pub fn on_choose(item: On<Choice>, mut clear_col: ResMut<ClearColor>) {
    match *item {
        Choice::SetColor(color) => clear_col.0 = color.into(),
    }
}
