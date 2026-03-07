use derive_more::Constructor;
use oc_individual::{IndividualIndex, Update, behavior::Behavior};
use oc_physics::{Force, MetersSeconds};
use oc_root::WORLD_HEIGHT;

use crate::{individual::Processor, utils::context::Context};

#[derive(Constructor)]
pub struct Move<'a> {
    ctx: &'a Context,
    i: IndividualIndex,
}

impl<'a> From<&'a Processor<'a>> for Move<'a> {
    fn from(value: &'a Processor) -> Self {
        let ctx = value.ctx;
        let i = value.i;

        Self { ctx, i }
    }
}

impl<'a> Move<'a> {
    pub fn read(&self) -> Vec<Update> {
        let world = self.ctx.state.world();
        let individual = world.individual(self.i);
        let (x, _): (u64, u64) = individual.tile.into();

        let (pulse, behavior) = match individual.behavior {
            Behavior::MovingNorth => {
                if x == 0 {
                    (
                        Some(Force::Translation([0.0, 1.0], MetersSeconds(0.5))),
                        Some(Behavior::MovingSouth),
                    )
                } else {
                    (None, None)
                }
            }
            Behavior::MovingSouth => {
                if x == WORLD_HEIGHT as u64 - 1 {
                    (
                        Some(Force::Translation([0.0, -1.0], MetersSeconds(0.5))),
                        Some(Behavior::MovingNorth),
                    )
                } else {
                    (None, None)
                }
            }
            Behavior::Idle => (
                Some(Force::Translation([0.0, 1.0], MetersSeconds(0.5))),
                Some(Behavior::MovingSouth),
            ),
        };

        tracing::trace!(name="indididual-move", i=?self.i, pulse=?pulse, behavior=?behavior);
        let mut updates = vec![];

        if let Some(pulse) = pulse {
            updates.push(Update::SetForces(vec![pulse]));
        }

        if let Some(behavior) = behavior {
            updates.push(Update::SetBehavior(behavior));
        }

        updates
    }
}
