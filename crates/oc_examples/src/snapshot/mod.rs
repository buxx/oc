use oc_geo::tile::WorldTileIndex;
use oc_individual::Individual;
use oc_projectile::Projectile;
use oc_root::TILES_COUNT;
use oc_world::{
    snapshot::Snapshot,
    tile::{Nature, Tile},
};

pub struct SnapshotBuilder {}

impl SnapshotBuilder {
    pub fn new() -> Self {
        Self {}
    }

    fn tiles(&self) -> Vec<Tile> {
        // TODO: settable generator
        (0..TILES_COUNT)
            .map(|i| Tile::new(WorldTileIndex(i as u64), Nature::ShortGrass, 0))
            .collect()
    }

    fn individuals(&self) -> Vec<Individual> {
        vec![]
    }

    fn projectiles(&self) -> Vec<Projectile> {
        vec![]
    }

    pub fn build(&self) -> Snapshot {
        let tiles = self.tiles();
        let individuals = self.individuals();
        let projectiles = self.projectiles();

        Snapshot {
            tiles,
            individuals,
            projectiles,
        }
    }
}
