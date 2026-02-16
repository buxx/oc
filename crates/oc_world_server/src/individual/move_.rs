use std::sync::Arc;

use derive_more::Constructor;
use oc_individual::{IndividualIndex, Update, behavior::Behavior};
use oc_physics::{Force, MetersSeconds};
use oc_root::WORLD_HEIGHT;

use crate::{individual::Processor, state::State};

#[derive(Constructor)]
pub struct Move {
    i: IndividualIndex,
    state: Arc<State>,
}

impl From<&Processor> for Move {
    fn from(value: &Processor) -> Self {
        let i = value.i;
        let state = value.state.clone();

        Self { i, state }
    }
}

impl Move {
    pub fn read(&self) -> Vec<Update> {
        let world = self.state.world();
        let individual = world.individual(self.i);
        let (x, _): (u64, u64) = individual.xy.into();

        let (pulse, behavior) = match individual.behavior {
            Behavior::MovingNorth => {
                if x == 0 {
                    (
                        Some(Force::Translation([1.0, 0.0], MetersSeconds(0.5))),
                        Some(Behavior::MovingSouth),
                    )
                } else {
                    (None, None)
                }
            }
            Behavior::MovingSouth => {
                if x == WORLD_HEIGHT as u64 - 1 {
                    (
                        Some(Force::Translation([-1.0, 0.0], MetersSeconds(0.5))),
                        Some(Behavior::MovingNorth),
                    )
                } else {
                    (None, None)
                }
            }
            Behavior::Idle => (
                Some(Force::Translation([1.0, 0.0], MetersSeconds(0.5))),
                Some(Behavior::MovingSouth),
            ),
        };

        let mut updates = vec![];

        if let Some(pulse) = pulse {
            updates.push(Update::PushForce(pulse));
        }

        if let Some(behavior) = behavior {
            updates.push(Update::UpdateBehavior(behavior));
        }

        updates
    }
}
