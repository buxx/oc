use derive_more::Constructor;
use oc_geo::tile::{TileXy, WorldTileIndex};
use oc_individual::{Individual, IndividualIndex};

pub mod tile;

#[derive(Constructor)]
pub struct World {
    tiles: Vec<tile::Tile>,
    individuals: Vec<Individual>,
}

impl World {
    pub fn tile(&self, xy: TileXy) -> Option<&tile::Tile> {
        self.tiles.get(WorldTileIndex::from(xy).0)
    }

    pub fn individuals(&self) -> &Vec<Individual> {
        &self.individuals
    }

    pub fn individual(&self, i: IndividualIndex) -> &Individual {
        &self.individuals[i.0 as usize]
    }

    pub fn individual_mut(&mut self, i: IndividualIndex) -> &mut Individual {
        &mut self.individuals[i.0 as usize]
    }
}
