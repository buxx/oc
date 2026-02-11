use std::sync::RwLockWriteGuard;

use derive_more::Constructor;
use oc_individual::behavior::Behavior;
use oc_utils::d2::Xy;
use oc_world::World;

use crate::{index::Indexes, individual::Processor};

pub enum Effect {
    Update(Xy, Behavior),
}

#[derive(Constructor)]
pub struct Apply<'a> {
    i: usize,
    world: RwLockWriteGuard<'a, World>,
    indexes: RwLockWriteGuard<'a, Indexes>,
}

impl<'a> From<&'a Processor> for Apply<'a> {
    fn from(value: &'a Processor) -> Self {
        let i = value.i;
        let world = value.world.write().unwrap();
        let indexes = value.indexes.write().unwrap();

        Self { i, world, indexes }
    }
}

impl<'a> Apply<'a> {
    pub fn apply(&mut self, effects: Vec<Effect>) {
        let individual = self.world.individual_mut(self.i);

        for effect in effects {
            match effect {
                Effect::Update(xy, behavior) => {
                    self.indexes.update_xy_individual(individual.xy, xy, self.i);
                    individual.xy = xy;

                    individual.behavior = behavior;
                }
            }
        }
    }
}
