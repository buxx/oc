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
    event: On<Pointer<Press>>,
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
        Name::new(format!("item-{}", &item.name())),
        item,
        Button,
        Node {
            // padding: UiRect::all(px(5)),
            ..default()
        },
        children![(Pickable::IGNORE, image,)],
    )
}

// pub fn on_over<T: ContextualMenu + Send + Sync + 'static>(
//     mut event: On<Pointer<Over>>,
//     mut items: Query<&mut ImageNode, With<T::Choices>>,
// ) {
//     let Ok(image) = items.get_mut(event.original_event_target()) else {
//         return;
//     };
//     event.propagate(false);
//     println!("IN");
// }

// pub fn on_out<T: ContextualMenu + Send + Sync + 'static>(
//     mut event: On<Pointer<Out>>,
//     mut items: Query<&mut ImageNode, With<T::Choices>>,
// ) {
//     let Ok(image) = items.get_mut(event.original_event_target()) else {
//         return;
//     };
//     event.propagate(false);
//     println!("OUT");
// }

// pub fn on_over<T>(mut event: On<Pointer<Over>>)
// where
//     T: ContextualMenu + Send + Sync + 'static,
// {
//     println!("in");
//     event.propagate(false);
// }

// pub fn on_out<T>(mut event: On<Pointer<Out>>)
// where
//     T: ContextualMenu + Send + Sync + 'static,
// {
//     println!("out");
//     event.propagate(false);
// }

pub fn on_over<T: ContextualMenu + Send + Sync + 'static>(
    event: On<Pointer<Over>>,
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
    event: On<Pointer<Out>>,
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
