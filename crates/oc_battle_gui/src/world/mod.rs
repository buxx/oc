use bevy::prelude::*;
use oc_geo::{
    region::{RegionXy, WorldRegionIndex},
    tile::{TileXy, WorldHeightIndex, WorldTileIndex},
};
use oc_individual::{
    Individual, IndividualIndex,
    squad::{Squad, SquadIndex},
};
use oc_physics::Physic;
use oc_root::{WcfgFrom, WcfgInto, WorldConfig};
#[cfg(feature = "debug")]
use oc_root::{physics::Meters, y::Y};
use oc_utils::d2::Xy;
use oc_world::tile::Tile;
use rustc_hash::FxHashMap;

use crate::{
    ingame::{WorldResumeEvent, behavior::SpawnSquadOrders, physics::ObjectId},
    states::GameConfig,
};

pub mod individual;
pub mod tile;

#[derive(Debug, Event)]
pub struct InsertTiles(pub WorldRegionIndex, pub Vec<(WorldTileIndex, Tile)>);

#[allow(unused)]
#[derive(Debug, Event)]
pub struct InsertedTiles(pub WorldRegionIndex);

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<World>()
            .add_observer(on_world_resume)
            .add_observer(tile::on_insert_tiles)
            .add_observer(tile::on_forgotten_region)
            .add_observer(individual::on_insert_individual)
            // .add_observer(individual::on_update_individual_position)
            .add_observer(individual::on_update_individual_physics)
            .add_observer(individual::on_forgotten_region);
    }
}

#[derive(Debug, Deref, DerefMut)]
pub struct Index<K: std::fmt::Debug, V: std::fmt::Debug>(
    FxHashMap<WorldRegionIndex, FxHashMap<K, V>>,
);

impl<K: std::fmt::Debug, V: std::fmt::Debug> Default for Index<K, V> {
    fn default() -> Self {
        Self(FxHashMap::default())
    }
}

// FIXME: improve perfs by using Partial and list index (like in previous projects)
#[derive(Resource, Default)]
pub struct World {
    pub individuals: Index<WorldTileIndex, Vec<(IndividualIndex, Individual)>>,
    pub individuals_refs: FxHashMap<IndividualIndex, (WorldRegionIndex, WorldTileIndex)>,
    pub tiles: Index<WorldTileIndex, Tile>,
    pub heights: Index<WorldHeightIndex, u8>,
    pub terrain: Option<oc_world::terrain::Terrain>,
    pub squads: Index<SquadIndex, Squad>,
    pub squads_refs: FxHashMap<SquadIndex, WorldRegionIndex>,
}

impl World {
    pub fn insert_tiles(&mut self, region: WorldRegionIndex, tiles: Vec<(WorldTileIndex, Tile)>) {
        let heights: Vec<(WorldHeightIndex, u8)> =
            tiles.iter().map(|(i, t)| ((*i).into(), t.z)).collect();
        self.tiles
            .entry(region)
            .and_modify(|tiles_| {
                // TODO: Damn, .clone() is pain here !
                for (i, tile) in tiles.clone().into_iter() {
                    tiles_.insert(i, tile);
                }
            })
            .or_insert(tiles.into_iter().collect());
        self.heights
            .entry(region)
            .and_modify(|heights_| {
                // TODO: Damn, .clone() is pain here !
                for (i, tile) in heights.clone().into_iter() {
                    heights_.insert(i, tile);
                }
            })
            .or_insert(heights.into_iter().collect());
    }

    pub fn remove_tiles(&mut self, region: WorldRegionIndex) {
        self.tiles.remove(&region);
    }

    pub fn insert_individual(
        &mut self,
        w: &WorldConfig,
        i: IndividualIndex,
        individual: Individual,
    ) {
        let position = individual.position(w);
        let tile_xy = TileXy(Xy(
            position[0] as u64 / w.geo_pixels_per_tile,
            position[1] as u64 / w.geo_pixels_per_tile,
        ));
        let tile: WorldTileIndex = tile_xy.into_(w);
        let region: WorldRegionIndex = tile.into_(w);

        // FIXME BS NOW: traced each seconds ?!
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

    pub fn remove_individual(&mut self, w: &WorldConfig, i: IndividualIndex, position: [f32; 3]) {
        let position = TileXy(Xy(position[0] as u64, position[1] as u64));
        let tile: WorldTileIndex = position.into_(w);
        let region: WorldRegionIndex = tile.into_(w);

        if let Some(tiles) = self.individuals.get_mut(&region)
            && let Some(individuals) = tiles.get_mut(&tile)
        {
            individuals.retain(|(i_, _)| *i_ != i);
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
                    .find_map(|(i_, individual)| (*i_ == i).then_some(individual))
            })
    }

    pub fn at(&self, w: &WorldConfig, tile: TileXy) -> Vec<(ObjectId, Box<&dyn Physic>)> {
        let region: WorldRegionIndex = tile.into_(w);
        let tile: WorldTileIndex = tile.into_(w);

        let mut objects = vec![];

        objects = self
            .individuals
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
            .unwrap_or_default();

        if let Some(tile_) = self.tiles.get(&region).and_then(|tiles| tiles.get(&tile)) {
            let tile_: Box<&dyn Physic> = Box::new(tile_);
            objects.push((ObjectId::Tile(tile), tile_));
        }

        objects
    }

    pub fn tile(&self, w: &WorldConfig, xy: TileXy) -> Option<&Tile> {
        let i: WorldTileIndex = xy.into_(w);
        let region: WorldRegionIndex = i.into_(w);
        self.tiles.get(&region).and_then(|tiles| tiles.get(&i))
    }

    pub fn tile_at(&self, w: &WorldConfig, point: &Vec2) -> Option<&Tile> {
        let xy = TileXy::from_([point.x, point.y], w);
        self.tile(w, xy)
    }

    #[cfg(feature = "debug")]
    pub fn tiles(&self) -> &Index<WorldTileIndex, Tile> {
        &self.tiles
    }

    #[cfg(feature = "debug")]
    pub fn heights(&self) -> &Index<WorldHeightIndex, u8> {
        &self.heights
    }

    #[cfg(feature = "debug")]
    pub fn point2d_to_point3d(
        &self,
        w: &WorldConfig,
        p: &Vec2,
        plus_z: Meters,
    ) -> Option<[f32; 3]> {
        use oc_root::WcfgFrom;

        let p = (p.x, p.y.to_world_y(w));
        let tile = TileXy::from_(p, w);
        let Some(tile) = self.tile(w, tile) else {
            return None;
        };
        let z = (tile.z as f32 * w.geo_meters_per_z.0 * w.geo_pixels_per_meters) + plus_z.pixels(w);
        let p = [p.0, p.1, z];
        Some(p)
    }
}

fn on_world_resume(
    event: On<WorldResumeEvent>,
    g: Res<GameConfig>,
    mut world: ResMut<World>,
    mut commands: Commands,
) {
    let Some(g) = &g.0 else { return };

    for (i, squad) in &event.0.squads {
        let position = squad.position;
        let tile_xy = TileXy::from_(position, &g.w);
        let region = RegionXy::from_(tile_xy, &g.w);
        let region = WorldRegionIndex::from_(region, &g.w);

        world
            .squads
            .entry(region)
            .or_insert_with(|| FxHashMap::default())
            .insert(*i, squad.clone());
        world.squads_refs.insert(*i, region);

        // FIXME BS NOW: debug here ?
        tracing::trace!(name="world-on-world-resume-trigger-spawn-squad-orders", i=?i, orders=?squad.orders);
        commands.trigger(SpawnSquadOrders(*i, squad.orders.clone()));
    }
}
