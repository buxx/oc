use bevy::prelude::*;

use crate::menu::contextual::ContextMenuItem;

pub fn context_item<E: Event + Clone + std::fmt::Debug>(item: ContextMenuItem<E>) -> impl Bundle {
    (
        Name::new(format!("item-{}", &item.text)),
        ContextMenuItem::new(item.text.clone(), item.event),
        Button,
        Node {
            padding: UiRect::all(px(5)),
            ..default()
        },
        children![(
            Pickable::IGNORE,
            Text::new(&item.text),
            TextFont {
                font_size: FontSize::Px(24.0),
                ..default()
            },
            TextColor(Color::WHITE),
        )],
    )
}

pub fn text_color_on_hover<T: std::fmt::Debug + Clone + Reflect>(
    color: Color,
) -> impl FnMut(On<Pointer<T>>, Query<&mut TextColor>, Query<&Children>) {
    move |mut event: On<Pointer<T>>,
          mut text_color: Query<&mut TextColor>,
          children: Query<&Children>| {
        let Ok(children) = children.get(event.original_event_target()) else {
            return;
        };
        event.propagate(false);

        // find the text among children and change its color
        for child in children.iter() {
            if let Ok(mut col) = text_color.get_mut(child) {
                col.0 = color;
            }
        }
    }
}
