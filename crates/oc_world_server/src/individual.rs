use std::{
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};

use derive_more::Constructor;
use oc_individual::behavior::Behavior;
use oc_root::{INDIVIDUAL_TICK_INTERVAL_US, WORLD_HEIGHT};
use oc_utils::d2::Xy;
use oc_world::World;

use crate::perf::Perf;

#[derive(Constructor)]
pub struct Processor {
    i: usize,
    perf: Arc<Perf>,
    world: Arc<RwLock<World>>,
}

impl Processor {
    pub fn run(self) {
        let mut last = Instant::now();

        loop {
            let elapsed = last.elapsed().as_micros() as u64;
            let wait = INDIVIDUAL_TICK_INTERVAL_US - elapsed;
            std::thread::sleep(Duration::from_micros(wait));

            let effects = self.read();
            if !effects.is_empty() {
                self.write(effects)
            }

            self.perf.incr();
            last = Instant::now();
        }
    }

    fn read(&self) -> Vec<Effect> {
        let world = self.world.read().unwrap();
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

    fn write(&self, effects: Vec<Effect>) {
        let mut world = self.world.write().unwrap();
        let individual = world.individual_mut(self.i);

        for effect in effects {
            match effect {
                Effect::Update(xy, behavior) => {
                    individual.xy = xy;
                    individual.behavior = behavior;
                }
            }
        }
    }
}

pub enum Effect {
    Update(Xy, Behavior),
}
