use bevy::prelude::*;

#[derive(Debug, Event)]
pub struct ToggleShowFormationPositions;

#[derive(Debug, Resource, Deref, DerefMut, Default)]
pub struct ShowFormationPositions(pub bool);

pub fn on_toggle_show_formation_positions(
    _: On<ToggleShowFormationPositions>,
    mut show: ResMut<ShowFormationPositions>,
) {
    show.0 = !show.0;
}
