use bevy::prelude::*;
use derive_more::Constructor;

use crate::menu::contextual::open::OpenContextualMenuEvent;

pub mod choice;
pub mod close;
pub mod item;
pub mod open;

#[derive(Debug)]
pub struct ContextualMenuPlugin<T: Send + Sync + 'static> {
    _marker: std::marker::PhantomData<T>,
}

impl<T: ContextualMenu + Send + Sync + 'static> Default for ContextualMenuPlugin<T> {
    fn default() -> Self {
        Self {
            _marker: Default::default(),
        }
    }
}

impl<T> Plugin for ContextualMenuPlugin<T>
where
    T: ContextualMenu + Send + Sync + 'static,
    for<'a> <T::ChoiceEvent as Event>::Trigger<'a>: Default,
{
    fn build(&self, app: &mut App) {
        app.add_observer(open::on_open::<T::OpenEvent, T::ChoiceEvent>)
            .add_observer(item::text_color_on_hover::<Out>(
                bevy::color::palettes::css::WHITE.into(),
            ))
            .add_observer(item::text_color_on_hover::<Over>(
                bevy::color::palettes::css::RED.into(),
            ))
            .add_observer(close::on_trigger_close_menus)
            .add_observer(|_: On<Pointer<Press>>, mut commands: Commands| {
                commands.trigger(close::CloseContextMenus);
            });
    }
}

#[derive(Component)]
pub struct ContextMenu;

#[derive(Component, Clone, Debug, Constructor)]
pub struct ContextMenuItem<E: Event + Clone + std::fmt::Debug> {
    text: String,
    event: E,
}

pub trait ContextualMenu {
    type OpenEvent: Event + OpenContextualMenuEvent<Self::ChoiceEvent>;
    type ChoiceEvent: Event + Clone + std::fmt::Debug;
}

#[derive(Debug, Clone, Constructor)]
pub struct Content<E: Event + Clone + std::fmt::Debug> {
    items: Vec<ContextMenuItem<E>>,
}
