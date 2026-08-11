use bevy::prelude::*;
use oc_root::{
    WcfgFrom,
    geo::{ScreenVec2, WorldVec2},
};
use oc_utils::{let_ok, let_some};

use crate::{
    menu::contextual::{Content, ContextMenu, choice::choose, item::context_item},
    states::GameConfig,
};

pub trait OpenContextualMenuEvent<I: Event + Clone + std::fmt::Debug> {
    fn position(&self) -> WorldVec2;
    fn content(&self) -> &Content<I>;
}

pub fn on_open<E, I>(
    event: On<E>,
    g: Res<GameConfig>,
    mut commands: Commands,
    camera: Single<(&Camera, &GlobalTransform)>,
) where
    E: Event + OpenContextualMenuEvent<I>,
    I: Event + Clone + std::fmt::Debug,
    for<'a> I::Trigger<'a>: Default,
{
    let_some!(g = &g.0, return);
    let (camera, camera_transform) = *camera;

    let position = event.position();
    let position = ScreenVec2::from_(position, &g.w);
    let position = camera.world_to_viewport(camera_transform, position.extend(0.).into());
    let_ok!(position = position, return);
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
