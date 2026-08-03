use bevy::prelude::*;
use oc_root::geo::ScreenVec2;

use crate::menu::contextual::{Content, ContextMenu, choice::choose, item::context_item};

pub trait OpenContextualMenuEvent<I: Event + Clone + std::fmt::Debug> {
    fn position(&self) -> ScreenVec2;
    fn content(&self) -> &Content<I>;
}

pub fn on_open<E, I>(event: On<E>, mut commands: Commands)
where
    E: Event + OpenContextualMenuEvent<I>,
    I: Event + Clone + std::fmt::Debug,
    for<'a> I::Trigger<'a>: Default,
{
    let position = event.position();
    let content = event.content().clone();

    commands
        .spawn((
            Name::new("context menu"),
            ContextMenu,
            Node {
                position_type: PositionType::Absolute,
                left: px(position.x),
                top: px(position.y),
                flex_direction: FlexDirection::Column,
                border_radius: BorderRadius::all(px(4)),
                ..default()
            },
            BorderColor::all(Color::BLACK),
            BackgroundColor(Color::linear_rgb(0.1, 0.1, 0.1)),
        ))
        .with_children(|parent| {
            content.items.into_iter().for_each(|item| {
                parent.spawn(context_item(item));
            })
        })
        .observe(choose::<I>);
}
