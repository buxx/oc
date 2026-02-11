use std::sync::Arc;

use derive_more::Constructor;
use oc_individual::behavior::Behavior;
use oc_root::WORLD_HEIGHT;
use oc_utils::d2::Xy;

use crate::{
    individual::{Processor, effect::Effect},
    state::State,
};

#[derive(Constructor)]
pub struct Move {
    i: usize,
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
    pub fn read(&self) -> Vec<Effect> {
        let world = self.state.world();
        let individual = world.individual(self.i);
        let (x, y): (usize, usize) = individual.xy.into();

        let (next_position, next_behavior) = match individual.behavior {
            Behavior::MovingNorth => {
                if x == 0 {
                    (individual.xy, Behavior::MovingSouth)
                } else {
                    (Xy(x - 1, y), Behavior::MovingNorth)
                }
            }
            Behavior::MovingSouth => {
                if x == WORLD_HEIGHT - 1 {
                    (individual.xy, Behavior::MovingNorth)
                } else {
                    (Xy(x + 1, y), Behavior::MovingSouth)
                }
            }
        };

        vec![Effect::Update(next_position, next_behavior)]
    }
}
