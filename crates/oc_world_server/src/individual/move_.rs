use derive_more::Constructor;
use oc_geo::tile::TileXy;
use oc_individual::{IndividualIndex, Update, behavior::Behavior};
use oc_physics::Force;
use oc_root::{Client, WcfgInto, physics::MetersSeconds};

use crate::{individual::Processor, utils::context::Context};

#[derive(Constructor)]
pub struct Move<'a, E: Client> {
    ctx: &'a Context<E>,
    i: IndividualIndex,
}

impl<'a, E: Client> From<&'a Processor<'a, E>> for Move<'a, E> {
    fn from(value: &'a Processor<E>) -> Self {
        let ctx = value.ctx;
        let i = value.i;

        Self { ctx, i }
    }
}

impl<'a, E: Client> Move<'a, E> {
    pub fn read(&self) -> Vec<Update> {
        let world = self.ctx.state.world();
        let individual = world.individual(self.i);
        let tile: TileXy = individual.tile.into_(&self.ctx.state.w);
        let (x, _): (u64, u64) = tile.into();

        let (pulse, behavior) = match individual.behavior {
            Behavior::MovingNorth => {
                if x == 0 {
                    (
                        Some(Force::Translation([0.0, 1.0, 0.0], MetersSeconds(0.5))),
                        Some(Behavior::MovingSouth),
                    )
                } else {
                    (None, None)
                }
            }
            Behavior::MovingSouth => {
                if x == self.ctx.state.w.world_height - 1 {
                    (
                        Some(Force::Translation([0.0, -1.0, 0.0], MetersSeconds(0.5))),
                        Some(Behavior::MovingNorth),
                    )
                } else {
                    (None, None)
                }
            }
            Behavior::Idle => {
                // Some(Force::Translation([0.0, 1.0, 0.0], MetersSeconds(0.5))),
                // Some(Behavior::MovingSouth),
                (None, None)
            }
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
