use derive_more::Constructor;
use oc_root::side::Side;
use oc_utils::polygon::Polygon;
use oc_world::{
    World,
    flag::{Flags, Ownership},
};

/// Compute flags ownership according to operational individuals inside flags
#[derive(Constructor)]
pub struct Processor<'a> {
    world: &'a World,
}

impl Processor<'_> {
    pub fn compute(&self) -> Flags {
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
}
