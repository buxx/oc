use oc_root::TILES_COUNT;
use oc_utils::d2::{Xy, XyIndex};
use oc_world::World;

pub struct Indexes {
    xy_individuals: Vec<Vec<usize>>,
}

impl Indexes {
    pub fn new(world: &World) -> Self {
        let mut xy_individuals = vec![vec![]; TILES_COUNT];

        for (i, individual) in world.individuals().iter().enumerate() {
            let tile_index = XyIndex::from(individual.xy).0;
            xy_individuals[tile_index].push(i);
        }

        Self { xy_individuals }
    }

    pub fn update_xy_individual(&mut self, before: Xy, now: Xy, i: usize) {
        let before_tile_index = XyIndex::from(before).0;
        let now_tile_index = XyIndex::from(now).0;

        self.xy_individuals[before_tile_index].retain(|i_| i_ != &i);
        self.xy_individuals[now_tile_index].push(now_tile_index);
    }
}
