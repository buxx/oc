use std::sync::{Arc, RwLock, RwLockReadGuard};

use derive_more::Constructor;
use oc_individual::behavior::Behavior;
use oc_root::WORLD_HEIGHT;
use oc_utils::d2::Xy;
use oc_world::World;

use crate::{
    index::Indexes,
    individual::{Processor, effect::Effect},
};

#[derive(Constructor)]
pub struct Move<'a> {
    i: usize,
    world: RwLockReadGuard<'a, World>,
    indexes: RwLockReadGuard<'a, Indexes>,
}

impl<'a> From<&'a Processor> for Move<'a> {
    fn from(value: &'a Processor) -> Self {
        let i = value.i;
        let world = value.world.read().unwrap();
        let indexes = value.indexes.read().unwrap();

        Self { i, world, indexes }
    }
}

impl<'a> Move<'a> {
    pub fn read(&self) -> Vec<Effect> {
        let individual = self.world.individual(self.i);
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
