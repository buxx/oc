use bevy::prelude::*;
use oc_geo::{
    region::WorldRegionIndex,
    tile::{TileXy, WorldTileIndex},
};
use oc_individual::{Individual, IndividualIndex};
use oc_physics::{Corps, Physic};
use oc_root::tile::Tile;
use oc_utils::d2::Xy;
use rustc_hash::FxHashMap;

use crate::ingame::physics::ObjectId;

pub mod individual;
pub mod tile;

#[derive(Debug, Event)]
pub struct InsertTiles(pub WorldRegionIndex, pub Vec<(WorldTileIndex, Tile)>);

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<World>()
            .add_observer(tile::on_insert_tiles)
            .add_observer(tile::on_forgotten_region)
            .add_observer(individual::on_insert_individual)
            // .add_observer(individual::on_update_individual_position)
            .add_observer(individual::on_update_individual_physics)
            .add_observer(individual::on_forgotten_region);
    }
}

#[derive(Deref, DerefMut)]
pub struct Index<K, V>(FxHashMap<WorldRegionIndex, FxHashMap<K, V>>);

impl<K, V> Default for Index<K, V> {
    fn default() -> Self {
        Self(FxHashMap::default())
    }
}

#[derive(Resource, Default)]
pub struct World {
    individuals: Index<WorldTileIndex, Vec<(IndividualIndex, Individual)>>,
    individuals_refs: FxHashMap<IndividualIndex, (WorldRegionIndex, WorldTileIndex)>,
    tiles: Index<WorldTileIndex, Tile>,
}

impl World {
    pub fn insert_tiles(&mut self, region: WorldRegionIndex, tiles: Vec<(WorldTileIndex, Tile)>) {
        self.tiles
            .entry(region)
            .and_modify(|tiles_| {
                // TODO: Damn, .clone() is pain here !
                for (i, tile) in tiles.clone().into_iter() {
                    tiles_.insert(i, tile);
                }
            })
            .or_insert(tiles.into_iter().collect());
    }

    pub fn remove_tiles(&mut self, region: WorldRegionIndex) {
        self.tiles.remove(&region);
    }

    pub fn insert_individual(&mut self, i: IndividualIndex, individual: Individual) {
        let position = individual.position();
        let tile_xy = TileXy(Xy(position[0] as u64, position[1] as u64));
        let tile: WorldTileIndex = tile_xy.into();
        let region: WorldRegionIndex = tile.into();

        tracing::trace!(name = "world-individual-insert", i=?i, region=?region, tile=?tile);

        let value = (i, individual);
        self.individuals
            .entry(region)
            .and_modify(|tiles| {
                tiles
                    .entry(tile)
                    .and_modify(|individuals| individuals.push(value.clone()))
                    .or_insert(vec![value.clone()]);
            })
            .or_insert(FxHashMap::from_iter(vec![(tile, vec![value])]));
        self.individuals_refs.insert(i, (region, tile));
    }

    pub fn remove_individual(&mut self, i: IndividualIndex, position: [f32; 2]) {
        let position = TileXy(Xy(position[0] as u64, position[1] as u64));
        let tile: WorldTileIndex = position.into();
        let region: WorldRegionIndex = tile.into();

        if let Some(tiles) = self.individuals.get_mut(&region) {
            if let Some(individuals) = tiles.get_mut(&tile) {
                individuals.retain(|(i_, _)| *i_ != i);
            }
        }
        self.individuals_refs.remove(&i);
    }

    pub fn remove_individuals(&mut self, region: WorldRegionIndex) {
        self.individuals.remove(&region);
    }

    pub fn get_individual(&self, i: IndividualIndex) -> Option<&Individual> {
        let Some((region, tile)) = self.individuals_refs.get(&i) else {
            return None;
        };

        self.individuals
            .get(region)
            .and_then(|region| region.get(tile))
            .and_then(|individuals| {
                individuals
                    .iter()
                    .find_map(|(i_, individual)| (*i_ == i).then(|| individual))
            })
    }

    pub fn at(&self, tile: TileXy) -> Vec<(ObjectId, Box<&dyn Physic>)> {
        let region: WorldRegionIndex = tile.into();
        let tile: WorldTileIndex = tile.into();

        self.individuals
            .get(&region)
            .and_then(|region| region.get(&tile))
            .map(|individuals| {
                individuals
                    .iter()
                    .map(|(i, individual)| {
                        let individual: Box<&dyn Physic> = Box::new(individual);
                        (ObjectId::Individual(*i), individual)
                    })
                    .collect::<Vec<(ObjectId, Box<&dyn Physic>)>>()
            })
            .unwrap_or_default()
    }

    pub fn tiles(&self) -> Vec<(&WorldTileIndex, &Tile)> {
        self.tiles
            .iter()
            .map(|(_, tiles)| tiles.iter())
            .flatten()
            .collect()
    }
}
