use std::sync::{Arc, RwLock, RwLockReadGuard};

use derive_more::Constructor;
use oc_individual::behavior::Behavior;
use oc_root::WORLD_HEIGHT;
use oc_utils::d2::Xy;
use oc_world::World;

use crate::{
    individual::{Processor, effect::Effect},
    perf::Perf,
};

#[derive(Constructor)]
pub struct Move {
    i: usize,
    perf: Arc<Perf>,
    world: Arc<RwLock<World>>,
}

impl From<&Processor> for Move {
    fn from(value: &Processor) -> Self {
        Self {
            i: value.i,
            perf: value.perf.clone(),
            world: value.world.clone(),
        }
    }
}

impl Move {
    pub fn read(&self, world: RwLockReadGuard<World>) -> Vec<Effect> {
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
