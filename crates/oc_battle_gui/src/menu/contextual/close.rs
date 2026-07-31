use bevy::prelude::*;

use crate::menu::contextual::ContextMenu;

#[derive(Event)]
pub struct CloseContextMenus;

pub fn on_trigger_close_menus(
    _event: On<CloseContextMenus>,
    mut commands: Commands,
    menus: Query<Entity, With<ContextMenu>>,
) {
    for e in menus.iter() {
        commands.entity(e).despawn();
    }
}
