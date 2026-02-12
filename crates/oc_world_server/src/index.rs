use oc_individual::IndividualIndex;
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

    pub fn update_individual_xy(&mut self, i: IndividualIndex, before: Xy, now: Xy) {
        let before_tile_index = XyIndex::from(before).0;
        let now_tile_index = XyIndex::from(now).0;

        self.xy_individuals[before_tile_index].retain(|i_| i_ != &(i.0 as usize));
        self.xy_individuals[now_tile_index].push(now_tile_index);
    }
}
