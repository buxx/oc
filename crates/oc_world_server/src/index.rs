use oc_geo::{
    region::{RegionXy, WorldRegionIndex},
    tile::{TileXy, WorldTileIndex},
};
use oc_individual::IndividualIndex;
use oc_root::{REGIONS_COUNT, TILES_COUNT};
use oc_world::World;

pub struct Indexes {
    tiles_individuals: Vec<Vec<IndividualIndex>>,
    regions_individuals: Vec<Vec<IndividualIndex>>,
}

impl Indexes {
    pub fn new(world: &World) -> Self {
        let mut tiles_individuals = vec![vec![]; TILES_COUNT];
        let mut regions_individuals = vec![vec![]; REGIONS_COUNT];

        for (i, individual) in world.individuals().iter().enumerate() {
            let tile: WorldTileIndex = individual.tile.into();
            let region: WorldRegionIndex = tile.into();

            tiles_individuals[tile.0].push(i.into());
            regions_individuals[region.0 as usize].push(i.into());
        }

        Self {
            tiles_individuals,
            regions_individuals,
        }
    }

    pub fn update_individual_tile(&mut self, i: IndividualIndex, now: TileXy, before: TileXy) {
        let before_tile: WorldTileIndex = before.into();
        self.tiles_individuals[before_tile.0].retain(|i_| *i_ != i);
        let now_tile: WorldTileIndex = now.into();
        self.tiles_individuals[now_tile.0].push(i);
    }

    pub fn update_individual_region(
        &mut self,
        i: IndividualIndex,
        now: RegionXy,
        before: RegionXy,
    ) {
        let before_region: WorldRegionIndex = before.into();
        self.regions_individuals[before_region.0 as usize].retain(|i_| *i_ != i);
        let now_region: WorldRegionIndex = now.into();
        self.regions_individuals[now_region.0 as usize].push(i);
    }

    pub fn region_individuals(&self, region: WorldRegionIndex) -> &Vec<IndividualIndex> {
        &self.regions_individuals[region.0 as usize]
    }
}
