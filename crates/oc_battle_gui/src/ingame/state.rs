use bevy::prelude::*;
use oc_individual::{IndividualIndex, order::Order, squad::SquadIndex};

#[derive(Debug, Resource, Default)]
#[cfg_attr(feature = "debug", derive(Clone))]
pub struct State {
    /// Squad under selection
    selected_squads: Vec<SquadIndex>,
    /// Individuals of selected squads
    selected_squads_individuals: Vec<IndividualIndex>,
    /// Individuals specially selected
    selected_individuals: Vec<IndividualIndex>,
    /// Orders which player is creating
    pending_orders: Vec<Order>,
}

impl State {
    pub fn selected_squads(&self) -> &[SquadIndex] {
        &self.selected_squads
    }

    #[allow(unused)]
    pub fn selected_squads_individuals(&self) -> &[IndividualIndex] {
        &self.selected_squads_individuals
    }

    #[allow(unused)]
    pub fn selected_individuals(&self) -> &[IndividualIndex] {
        &self.selected_individuals
    }

    /// Entrypoint for selection update. Only way to update selection which require set all values at once.
    pub fn update_selected(
        &mut self,
        squads: Vec<SquadIndex>,
        squads_individuals: Vec<IndividualIndex>,
        individuals: Vec<IndividualIndex>,
    ) {
        self.selected_squads = squads;
        self.selected_squads_individuals = squads_individuals;
        self.selected_individuals = individuals;
    }

    pub fn pending_orders(&self) -> &[Order] {
        &self.pending_orders
    }

    pub fn clear_pending_orders(&mut self) {
        self.pending_orders.clear()
    }

    pub fn push_pending_orders(&mut self, value: Order) {
        self.pending_orders.push(value)
    }
}
