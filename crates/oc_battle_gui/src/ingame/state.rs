use bevy::prelude::*;
use oc_individual::{order::Order, squad::SquadIndex};

#[derive(Debug, Resource, Default)]
#[cfg_attr(feature = "debug", derive(Clone))]
pub struct State {
    selected_squads: Vec<SquadIndex>,
    pending_orders: Vec<Order>,
}

impl State {
    #[allow(unused)]
    pub fn selected_squads(&self) -> &[SquadIndex] {
        &self.selected_squads
    }

    pub fn set_selected_squads(&mut self, selected_squads: Vec<SquadIndex>) {
        self.selected_squads = selected_squads;
    }

    pub fn pending_orders(&self) -> &[Order] {
        &self.pending_orders
    }

    pub fn pending_orders_mut(&mut self) -> &mut Vec<Order> {
        &mut self.pending_orders
    }
}
