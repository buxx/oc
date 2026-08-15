use std::ops::{Deref, DerefMut};

use oc_geo::region::Region;
use oc_geo::tile::TileXy;
use oc_geo::{region::WorldRegionIndex, tile::WorldTileIndex};
use oc_individual::squad::SquadIndex;
use oc_individual::{
    INDIVIDUAL_STAND_UP_VOLUME_HEIGHT, INDIVIDUAL_STAND_UP_VOLUME_WIDTH, IndividualIndex,
};
use oc_projectile::Projectile;
use oc_projectile::ProjectileId;
use oc_root::side::Side;
use oc_root::{WcfgFrom, WcfgInto, WorldConfig};
use oc_utils::d2::{Xy, shape_cover_tiles};
use oc_world::World;

use crate::physics;

pub struct SizedIndex<T>(Vec<Vec<T>>);

impl<T: std::clone::Clone> SizedIndex<T> {
    pub fn new(size: usize) -> Self {
        Self(vec![vec![]; size])
    }
}

impl<T> Deref for SizedIndex<T> {
    type Target = Vec<Vec<T>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for SizedIndex<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub struct Indexes {
    tiles_individuals: SizedIndex<IndividualIndex>,
    regions_individuals: SizedIndex<IndividualIndex>,
    regions_projectiles: SizedIndex<ProjectileId>,
    individuals_squad: Vec<SquadIndex>,
    side_a_individuals: Vec<IndividualIndex>,
    side_b_individuals: Vec<IndividualIndex>,
}

impl Indexes {
    pub fn new(world: &World, w: &WorldConfig) -> Self {
        let individuals = world.individuals();

        let mut tiles_individuals = SizedIndex::new(world.w.tiles_count as usize);
        let mut regions_individuals = SizedIndex::new(world.w.regions_count as usize);
        let mut regions_projectiles = SizedIndex::new(world.w.regions_count as usize);
        let mut individuals_squad = Vec::with_capacity(world.individuals.len());
        let side_a_count = individuals.iter().filter(|i| i.side == Side::A).count();
        let mut side_a_individuals = Vec::with_capacity(side_a_count);
        let side_b_count = individuals.iter().filter(|i| i.side == Side::B).count();
        let mut side_b_individuals = Vec::with_capacity(side_b_count);

        for (i, individual) in individuals.iter().enumerate() {
            let position = individual.position;
            let tile: WorldTileIndex = individual.tile;
            let region: WorldRegionIndex = tile.into_(&world.w);

            for tile_ in shape_cover_tiles(
                [position.x, position.y],
                INDIVIDUAL_STAND_UP_VOLUME_WIDTH.pixels(w),
                INDIVIDUAL_STAND_UP_VOLUME_HEIGHT.pixels(w),
                w.geo_pixels_per_tile as f32,
                w.geo_pixels_per_tile as f32,
            ) {
                let tile_ = TileXy(Xy(tile_[0] as u64, tile_[1] as u64));
                if tile_.0.0 >= w.world_width || tile_.0.1 >= w.world_height {
                    continue;
                }
                let tile_ = WorldTileIndex::from_(tile_, w);
                tiles_individuals[tile_.0 as usize].push(i.into());

                match individual.side {
                    Side::A => side_a_individuals.push(IndividualIndex(i as u64)),
                    Side::B => side_b_individuals.push(IndividualIndex(i as u64)),
                }
            }

            regions_individuals[region.0 as usize].push(i.into());

            match world.squads.iter().enumerate().find(|(_, squad)| {
                squad
                    .members
                    .iter()
                    .find(|member_i| member_i.0 == i as u64)
                    .is_some()
            }) {
                Some((squad_i, _)) => {
                    individuals_squad.push(SquadIndex(squad_i as u64));
                }
                None => panic!("There is not squad owning member {i:?}"),
            }
        }

        for (id, projectile) in world.projectiles() {
            let tile: WorldTileIndex = projectile.tile();
            let region: WorldRegionIndex = tile.into_(&world.w);

            regions_projectiles[region.0 as usize].push(*id);
        }

        Self {
            tiles_individuals,
            regions_individuals,
            regions_projectiles,
            individuals_squad,
            side_a_individuals,
            side_b_individuals,
        }
    }

    pub fn insert_projectile(&mut self, id: ProjectileId, projectile: &Projectile) {
        self.update_projectile_region(id, projectile.region(), projectile.region());
    }

    pub fn remove_projectile(&mut self, id: &ProjectileId, projectile: &Projectile) {
        let region = projectile.region();
        self.regions_projectiles[region.0 as usize].retain(|p| p != id);
    }

    fn update_individual_position(
        &mut self,
        i: IndividualIndex,
        now: [f32; 2],
        // TODO: maintain individual positions index to compute it internally ?
        before: [f32; 2],
        w: &WorldConfig,
    ) {
        for before in shape_cover_tiles(
            before,
            INDIVIDUAL_STAND_UP_VOLUME_WIDTH.pixels(w),
            INDIVIDUAL_STAND_UP_VOLUME_HEIGHT.pixels(w),
            w.geo_pixels_per_tile as f32,
            w.geo_pixels_per_tile as f32,
        ) {
            let before = TileXy(Xy(before[0] as u64, before[1] as u64));
            if before.0.0 >= w.world_width || before.0.1 >= w.world_height {
                continue;
            }
            let before = WorldTileIndex::from_(before, w);
            self.tiles_individuals[before.0 as usize].retain(|i_| *i_ != i);
        }

        for now in shape_cover_tiles(
            now,
            INDIVIDUAL_STAND_UP_VOLUME_WIDTH.pixels(w),
            INDIVIDUAL_STAND_UP_VOLUME_HEIGHT.pixels(w),
            w.geo_pixels_per_tile as f32,
            w.geo_pixels_per_tile as f32,
        ) {
            let now = TileXy(Xy(now[0] as u64, now[1] as u64));
            if now.0.0 >= w.world_width || now.0.1 >= w.world_height {
                continue;
            }
            let now = WorldTileIndex::from_(now, w);
            self.tiles_individuals[now.0 as usize].push(i);
        }
    }

    fn update_individual_region(
        &mut self,
        i: IndividualIndex,
        now: WorldRegionIndex,
        before: WorldRegionIndex,
    ) {
        self.regions_individuals[before.0 as usize].retain(|i_| *i_ != i);
        self.regions_individuals[now.0 as usize].push(i);
    }

    fn update_projectile_region(
        &mut self,
        id: ProjectileId,
        now: WorldRegionIndex,
        before: WorldRegionIndex,
    ) {
        self.regions_projectiles[before.0 as usize].retain(|id_| *id_ != id);
        self.regions_projectiles[now.0 as usize].push(id);
    }

    pub fn tile_individuals(&self, tile: WorldTileIndex) -> &Vec<IndividualIndex> {
        &self.tiles_individuals[tile.0 as usize]
    }

    pub fn region_individuals(&self, region: WorldRegionIndex) -> &Vec<IndividualIndex> {
        &self.regions_individuals[region.0 as usize]
    }

    pub fn region_projectiles(&self, region: WorldRegionIndex) -> &Vec<ProjectileId> {
        &self.regions_projectiles[region.0 as usize]
    }

    pub fn individual_squad(&self, individual: IndividualIndex) -> SquadIndex {
        self.individuals_squad[individual.0 as usize]
    }

    pub fn react(&mut self, effect: Effect, w: &WorldConfig) {
        match effect {
            Effect::Individual(i, effect) => match effect {
                IndividualEffect::Physic(effect) => match effect {
                    physics::Effect::Position { before, after } => {
                        let now = [after.x, after.y];
                        let before = [before.x, before.y];
                        self.update_individual_position(i, now, before, w)
                    }
                    physics::Effect::Tile {
                        _before: _,
                        _after: _,
                    } => {}
                    physics::Effect::Region { before, after } => {
                        self.update_individual_region(i, after, before)
                    }
                },
            },
            Effect::Projectile(i, effect) => match effect {
                ProjectileEffect::Physic(effect) => match effect {
                    physics::Effect::Position {
                        before: _,
                        after: _,
                    }
                    | physics::Effect::Tile {
                        _before: _,
                        _after: _,
                    } => {}
                    physics::Effect::Region { before, after } => {
                        self.update_projectile_region(i, after, before)
                    }
                },
            },
        }
    }

    pub fn side_a_individuals(&self) -> &[IndividualIndex] {
        &self.side_a_individuals
    }

    pub fn side_b_individuals(&self) -> &[IndividualIndex] {
        &self.side_b_individuals
    }
}

pub enum Effect {
    Individual(IndividualIndex, IndividualEffect),
    Projectile(ProjectileId, ProjectileEffect),
}

pub enum IndividualEffect {
    Physic(physics::Effect),
}

pub enum ProjectileEffect {
    Physic(physics::Effect),
}

pub trait IntoIndexEffect<T> {
    fn into_index_effect(&self, value: T) -> Effect;
}

#[cfg(test)]
mod tests {
    use ::tests::{individual::TestIndividual, world::TestWorld};
    use oc_root::{WorldConfig, geo::WorldVec3, physics::Meters};

    use super::*;

    #[test]
    fn test_indexes_individuals() {
        // Given
        let w = WorldConfig::new(2, 2, Meters(0.1));
        let individual = TestIndividual::builder()
            .position(WorldVec3::new(4., 4., 0.))
            .build()
            .make(&w);
        let world = TestWorld::builder()
            .individuals(vec![individual])
            .build()
            .make(&w);

        // When
        let indexes = Indexes::new(&world, &w);

        // Then
        let x0y0 = indexes.tile_individuals(WorldTileIndex(0));
        let x1y0 = indexes.tile_individuals(WorldTileIndex(1));
        let x0y1 = indexes.tile_individuals(WorldTileIndex(2));
        let x1y1 = indexes.tile_individuals(WorldTileIndex(3));

        assert_eq!(x0y0, &vec![IndividualIndex(0)]);
        assert_eq!(x1y0, &vec![IndividualIndex(0)]);
        assert_eq!(x0y1, &vec![IndividualIndex(0)]);
        assert_eq!(x1y1, &vec![IndividualIndex(0)]);
    }

    #[test]
    fn test_indexes_individuals_removed() {
        // Given
        let w = WorldConfig::new(2, 2, Meters(0.1));
        let individual = TestIndividual::builder()
            .position(WorldVec3::new(4., 4., 0.))
            .build()
            .make(&w);
        let world = TestWorld::builder()
            .individuals(vec![individual])
            .build()
            .make(&w);
        let mut indexes = Indexes::new(&world, &w);

        // When
        indexes.update_individual_position(IndividualIndex(0), [9., 9.], [4., 4.], &w);

        // Then
        let x0y0 = indexes.tile_individuals(WorldTileIndex(0));
        let x1y0 = indexes.tile_individuals(WorldTileIndex(1));
        let x0y1 = indexes.tile_individuals(WorldTileIndex(2));
        let x1y1 = indexes.tile_individuals(WorldTileIndex(3));

        assert_eq!(x0y0, &Vec::<IndividualIndex>::new());
        assert_eq!(x1y0, &Vec::<IndividualIndex>::new());
        assert_eq!(x0y1, &Vec::<IndividualIndex>::new());
        assert_eq!(x1y1, &vec![IndividualIndex(0)]);
    }
}
