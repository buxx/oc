use bevy::prelude::*;
use oc_utils::let_ok;
use strum::IntoEnumIterator;

use crate::menu::contextual::{ContextualMenu, close::CloseContextMenu};

pub trait Choice<E: Event + Clone + std::fmt::Debug>: IntoEnumIterator + Component {
    fn name(&self) -> String;
    fn event(&self) -> E;
    fn idle(&self) -> Rect;
    fn hover(&self) -> Rect;
}

pub fn choose<T: ContextualMenu + Send + Sync + 'static>(
    event: On<PointerPress>,
    items: Query<&T::Choices>,
    mut commands: Commands,
) where
    for<'a> <T::ChoiceEvent as Event>::Trigger<'a>: Default,
{
    let target = event.original_event_target();

    if let Ok(item) = items.get(target) {
        commands.trigger(item.event());
        commands.trigger(CloseContextMenu::<T>::default());
    }
}

pub fn context_item<T: Choice<E>, E: Event + Clone + std::fmt::Debug>(
    item: T,
    image: ImageNode,
) -> impl Bundle {
    (
        Name::new(format!("item-{}", item.name())),
        item,
        Node {
            // padding: UiRect::all(px(5)),
            ..default()
        },
        children![(Pickable::IGNORE, image,)],
    )
}

pub fn on_over<T: ContextualMenu + Send + Sync + 'static>(
    event: On<PointerOver>,
    items: Query<(&T::Choices, &Children)>,
    mut images: Query<&mut ImageNode>,
) {
    let target = event.event_target();

    let_ok!((item, children) = items.get(target), return);

    for child in children.iter() {
        if let Ok(mut image) = images.get_mut(child) {
            image.rect = Some(item.hover());
            break;
        }
    }
}

pub fn on_out<T: ContextualMenu + Send + Sync + 'static>(
    event: On<PointerOut>,
    items: Query<(&T::Choices, &Children)>,
    mut images: Query<&mut ImageNode>,
) {
    let target = event.event_target();

    let_ok!((item, children) = items.get(target), return);

    for child in children.iter() {
        if let Ok(mut image) = images.get_mut(child) {
            image.rect = Some(item.idle());
            break;
        }
    }
}
