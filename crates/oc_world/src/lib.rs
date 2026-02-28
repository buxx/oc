use derive_more::Constructor;
use oc_geo::tile::{TileXy, WorldTileIndex};
use oc_individual::{Individual, IndividualIndex};

use crate::meta::Meta;

pub mod cache;
pub mod load;
pub mod map;
pub mod meta;
pub mod tile;

#[derive(Constructor)]
pub struct World {
    meta: Meta,
    tiles: Vec<tile::Tile>,
    individuals: Vec<Individual>,
}

impl World {
    pub fn tile(&self, xy: TileXy) -> Option<&tile::Tile> {
        self.tiles.get(WorldTileIndex::from(xy).0)
    }

    pub fn individuals(&self) -> &[Individual] {
        &self.individuals
    }

    pub fn individual(&self, i: IndividualIndex) -> &Individual {
        &self.individuals[i.0 as usize]
    }

    pub fn individual_mut(&mut self, i: IndividualIndex) -> &mut Individual {
        &mut self.individuals[i.0 as usize]
    }
}
