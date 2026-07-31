use bevy::prelude::*;

use crate::menu::contextual::{ContextMenuItem, close::CloseContextMenus};

pub fn choose<E>(
    event: On<Pointer<Press>>,
    items: Query<&ContextMenuItem<E>>,
    mut commands: Commands,
) where
    E: Event + Clone + std::fmt::Debug + Send + Sync + 'static,
    for<'a> E::Trigger<'a>: Default,
{
    let target = event.original_event_target();

    if let Ok(item) = items.get(target) {
        commands.trigger(item.event.clone());
        commands.trigger(CloseContextMenus);
    }
}
