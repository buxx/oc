use derive_more::Constructor;
use oc_individual::{
    Individual,
    squad::{SquadIndex, Update},
};
use oc_world::World;

use crate::{index::Indexes, runner};

pub mod update;

#[derive(Constructor)]
pub struct Processor<'a> {
    world: &'a World,
    _index: &'a Indexes,
    i: SquadIndex,
}

impl<'a> Processor<'a> {
    pub fn step(self) -> Vec<runner::update::Update> {
        tracing::trace!(name="squad-step", i=?self.i);

        let squad = self.world.squad(self.i);
        let leader = self.world.individual(squad.leader());
        let members: Vec<&Individual> = squad
            .members
            .iter()
            .map(|i| self.world.individual(*i))
            .collect();

        let position = leader.position.into();
        let actives = members
            .into_iter()
            .filter(|m| m.can_follow_orders())
            .count();
        let updates = vec![
            runner::update::Update::UpdateSquad(self.i, Update::SetPosition(position)),
            runner::update::Update::UpdateSquad(self.i, Update::SetActives(actives as u8)),
        ];

        tracing::trace!(name = "squad-step-updates", i=?self.i, updates=?updates);
        updates
    }
}
