use oc_geo::{
    region::WorldRegionIndex,
    tile::{TileXy, WorldTileIndex},
};
use oc_individual::IndividualIndex;
use oc_root::{REGIONS_COUNT, TILES_COUNT};
use oc_world::World;

pub struct Indexes {
    xy_individuals: Vec<Vec<IndividualIndex>>,
    regions_individuals: Vec<Vec<IndividualIndex>>,
}

impl Indexes {
    pub fn new(world: &World) -> Self {
        let mut xy_individuals = vec![vec![]; TILES_COUNT];
        let mut regions_individuals = vec![vec![]; REGIONS_COUNT];

        for (i, individual) in world.individuals().iter().enumerate() {
            let tile: WorldTileIndex = individual.xy.into();
            let region: WorldRegionIndex = tile.into();

            xy_individuals[tile.0].push(i.into());
            regions_individuals[region.0].push(i.into());
        }

        Self {
            xy_individuals,
            regions_individuals,
        }
    }

    pub fn update_individual_xy(&mut self, i: IndividualIndex, now: TileXy, before: TileXy) {
        let before_tile: WorldTileIndex = before.into();
        let before_region: WorldRegionIndex = before_tile.into();

        self.xy_individuals[before_tile.0].retain(|i_| *i_ != i);
        self.regions_individuals[before_region.0].retain(|i_| *i_ != i);

        let now_tile: WorldTileIndex = now.into();
        let now_region: WorldRegionIndex = now_tile.into();

        self.xy_individuals[now_tile.0].push(i);
        self.regions_individuals[now_region.0].push(i);
    }
}
