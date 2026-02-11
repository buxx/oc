use std::sync::Arc;

use derive_more::Constructor;
use oc_individual::behavior::Behavior;
use oc_utils::d2::Xy;

use crate::{individual::Processor, state::State};

pub enum Effect {
    Update(Xy, Behavior),
}

#[derive(Constructor)]
pub struct Apply {
    i: usize,
    state: Arc<State>,
}

impl From<&Processor> for Apply {
    fn from(value: &Processor) -> Self {
        let i = value.i;
        let state = value.state.clone();

        Self { i, state }
    }
}

impl Apply {
    pub fn apply(&mut self, effects: Vec<Effect>) {
        let mut world = self.state.world_mut();
        let mut indexes = self.state.indexes_mut();
        let individual = world.individual_mut(self.i);

        for effect in effects {
            match effect {
                Effect::Update(xy, behavior) => {
                    indexes.update_xy_individual(individual.xy, xy, self.i);
                    individual.xy = xy;

                    individual.behavior = behavior;
                }
            }
        }
    }
}
