use bevy::prelude::*;
use oc_utils::let_some;

use crate::ingame::input::left_click::LeftClick;
use crate::ingame::selected::{Select, Selected};
use crate::ingame::state::State;
use crate::world::World;

/// Un select all. Observer which select must stop propagation to avoid execute this observer.
pub fn unselect(_: On<Pointer<Click>>, mut state: ResMut<State>) {
    state.update_selected(vec![], vec![], vec![]);
}

pub fn on_select(
    event: On<Select>,
    mode: Res<LeftClick>,
    world: Res<World>,
    mut state: ResMut<State>,
) {
    // FIXME BS NOW: (https://github.com/bevyengine/bevy/pull/22602)
    // observer can now run_if ! Use LeftClick state
    // FIXME BS NOW: additionally, use state (and set run_if everywhere needed) to know if cursor is
    // in bevy window (must disable lot of things in that case)
    if !mode.0.is_select() {
        return;
    }

    match event.0 {
        Selected::Individual(i) => select_individual(i, &world, &mut state),
    }
}

fn select_individual(i: oc_individual::IndividualIndex, world: &World, state: &mut State) {
    let_some!((squad, _) = world.individual_squad(i), return);
    let squads = vec![squad];
    let_some!(squad = world.squad(squad), return);
    state.update_selected(squads, squad.members.clone(), vec![i]);
}
