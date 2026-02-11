use std::sync::RwLockWriteGuard;

use oc_individual::behavior::Behavior;
use oc_utils::d2::Xy;
use oc_world::World;

pub enum Effect {
    Update(Xy, Behavior),
}

pub fn apply(i: usize, mut world: RwLockWriteGuard<World>, effects: Vec<Effect>) {
    let individual = world.individual_mut(i);

    for effect in effects {
        match effect {
            Effect::Update(xy, behavior) => {
                individual.xy = xy;
                individual.behavior = behavior;
            }
        }
    }
}
