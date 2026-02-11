use derive_more::Constructor;
use oc_individual::Individual;
use oc_root::{INDIVIDUALS_COUNT, TILES_COUNT};
use oc_utils::d2::{Xy, XyIndex};

pub mod tile;

#[derive(Constructor)]
pub struct World {
    tiles: [tile::Tile; TILES_COUNT],
    individuals: [Individual; INDIVIDUALS_COUNT],
}

impl World {
    pub fn tile(&self, xy: Xy) -> Option<&tile::Tile> {
        self.tiles.get(XyIndex::from(xy).0)
    }

    pub fn individual(&self, i: usize) -> &Individual {
        &self.individuals[i]
    }

    pub fn individual_mut(&mut self, i: usize) -> &mut Individual {
        &mut self.individuals[i]
    }
}
