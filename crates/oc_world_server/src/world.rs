use crate::runner::update::Update;
use derive_more::Constructor;
use oc_root::{battle::BattlePhase, side::Side};
use oc_utils::polygon::Polygon;
use oc_world::{
    World,
    flag::{Flags, Ownership},
    morale::Morale,
};

#[derive(Constructor)]
pub struct Processor<'a> {
    world: &'a World,
}

impl Processor<'_> {
    pub fn compute(&self) -> Vec<Update> {
        let mut updates = vec![
            Update::UpdateFlags(self.compute_flags()),
            Update::UpdateMorale(self.compute_morale()),
        ];

        updates.extend(self.time());
        updates
    }

    fn time(&self) -> Vec<Update> {
        let mut updates = vec![];

        if let Some(elapsed) = self.world.elapsed() {
            updates.push(Update::UpdateElapsed(elapsed, self.world.duration));
            if elapsed >= self.world.duration {
                updates.push(Update::SetBattlePhase(BattlePhase::End));
            }
        }

        updates
    }

    fn compute_flags(&self) -> Flags {
        let mut flags = self.world.flags.clone();

        for (flag, ownership) in flags.iter_mut() {
            let mut side_a = false;
            let mut side_b = false;

            for individual in self.world.individuals() {
                if !matches!(individual.status, oc_individual::Status::Operational) {
                    continue;
                }
                if !flag.contains(individual.position.into()) {
                    continue;
                }

                match individual.side {
                    Side::A => side_a = true,
                    Side::B => side_b = true,
                }
                if side_a && side_b {
                    break;
                }
            }

            match (side_a, side_b) {
                (true, true) => *ownership = Ownership::Both,
                (true, false) => *ownership = Ownership::A,
                (false, true) => *ownership = Ownership::B,
                (false, false) => {}
            }
        }

        flags
    }

    fn compute_morale(&self) -> Morale {
        let percent = |side: Side| {
            let individuals = self
                .world
                .individuals()
                .iter()
                .filter(|individual| individual.side == side);
            let total = individuals.clone().count();
            let operational = individuals
                .filter(|individual| {
                    matches!(individual.status, oc_individual::Status::Operational)
                })
                .count();

            match total {
                0 => 0.,
                _ => operational as f32 / total as f32 * 100.,
            }
        };

        Morale {
            a: percent(Side::A),
            b: percent(Side::B),
        }
    }
}
