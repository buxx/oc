use oc_individual::Individual;
use oc_world::tile::Tile;

pub trait IndividualsGenerator {
    fn individuals(&self, tiles: &Vec<Tile>) -> Vec<Individual>;
}
