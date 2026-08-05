use bevy::prelude::*;
use oc_utils::bevy::EntityMapping;
use oc_utils::{let_ok, let_some};

use crate::entity::individual::IndividualIndex;
use crate::ingame::input::left_click::LeftClick;
use crate::ingame::state::State;
use crate::utils::selected::Selected;
use crate::world::World;

#[derive(Debug, Clone, Event)]
pub enum Select {
    Individual(oc_individual::IndividualIndex),
}

/// Un select all. Observer which select must stop propagation to avoid execute this observer.
pub fn unselect(_: On<Pointer<Click>>, mut state: ResMut<State>) {
    state.update_selected(vec![], vec![], vec![]);
}

pub fn on_select(
    event: On<Select>,
    mode: Res<LeftClick>,
    world: Res<World>,
    mut state: ResMut<State>,
    individuals: Res<EntityMapping<oc_individual::IndividualIndex>>,
    mut query: Query<(&IndividualIndex, &mut Selected)>,
) {
    // FIXME BS NOW: (https://github.com/bevyengine/bevy/pull/22602)
    // observer can now run_if ! Use LeftClick state
    // FIXME BS NOW: additionally, use state (and set run_if everywhere needed) to know if cursor is
    // in bevy window (must disable lot of things in that case)
    if !mode.0.is_select() {
        return;
    }

    match *event {
        Select::Individual(i) => select_individual(i, &world, &mut state, &individuals, &mut query),
    }
}

fn select_individual(
    i: oc_individual::IndividualIndex,
    world: &World,
    state: &mut State,
    individuals: &EntityMapping<oc_individual::IndividualIndex>,
    query: &mut Query<(&IndividualIndex, &mut Selected)>,
) {
    let_some!((squad, _) = world.individual_squad(i), return);
    let squads = vec![squad];
    let_some!(squad = world.squad(squad), return);

    state.update_selected(squads, squad.members.clone(), vec![i]);
    for individual in &squad.members {
        let_some!(individual = individuals.get(&individual), continue);
        let_ok!((_, mut selected) = query.get_mut(*individual), continue);
        selected.0 = true;
    }
}
