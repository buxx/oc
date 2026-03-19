use bevy::prelude::*;
use oc_geo::UpdateGeo;
use oc_geo::tile::TileXy;
use oc_geo::{region::WorldRegionIndex, tile::WorldTileIndex};
use oc_physics::update::bevy::SetPositionEvent;
use oc_physics::{Physic, UpdatePhysic};
use oc_utils::d2::Xy;
use rustc_hash::FxHashMap;

use crate::ingame::input::individual::{InsertIndividualEvent, UpdateIndividualPhysicsEvent};
use crate::ingame::region::ForgottenRegion;
use oc_individual::IndividualIndex;

// FIXME: use an Index (based on Vec) like server do (check not using too much RAM ...)
// TODO: maybe need improve perf on access tiles. Think about a Vec (with size of world tiles ?) containing references
// or position of tiles in gui vec ?
// #[derive(Debug, Resource, Default, Deref, DerefMut)]
// pub struct Individuals(
//     pub FxHashMap<WorldRegionIndex, FxHashMap<WorldTileIndex, Vec<IndividualIndex>>>,
// );

// impl Individuals {
//     pub fn insert(&mut self, i: IndividualIndex, position: [f32; 2]) {
//         let position = TileXy(Xy(position[0] as u64, position[1] as u64));
//         let tile: WorldTileIndex = position.into();
//         let region: WorldRegionIndex = tile.into();

//         tracing::trace!(name = "world-individual-insert", i=?i, region=?region, tile=?tile);

//         self.entry(region)
//             .and_modify(|tiles| {
//                 tiles
//                     .entry(tile)
//                     .and_modify(|individuals| individuals.push(i))
//                     .or_insert(vec![i]);
//             })
//             .or_insert(FxHashMap::from_iter(vec![(tile, vec![i])]));
//     }

//     pub fn remove(&mut self, i: IndividualIndex, position: [f32; 2]) {
//         let position = TileXy(Xy(position[0] as u64, position[1] as u64));
//         let tile: WorldTileIndex = position.into();
//         let region: WorldRegionIndex = tile.into();

//         if let Some(tiles) = self.get_mut(&region) {
//             if let Some(individuals) = tiles.get_mut(&tile) {
//                 individuals.retain(|i_| *i_ != i);
//             }
//         }
//     }

//     // pub fn at(&self, xy: &Xy) -> &Vec<IndividualIndex> {
//     //     let position = TileXy(*xy);
//     //     let tile: WorldTileIndex = position.into();
//     //     let region: WorldRegionIndex = tile.into();

//     //     if let Some(tiles) = self.get(&region) {
//     //         if let Some(individuals) = tiles.get(&tile) {
//     //             return individuals;
//     //         }
//     //     }

//     //     static EMPTY: Vec<IndividualIndex> = vec![];
//     //     &EMPTY
//     // }
// }

pub fn on_insert_individual(insert: On<InsertIndividualEvent>, mut index: ResMut<super::World>) {
    let i = insert.0;
    let individual = &insert.1;

    index.insert_individual(i, individual.clone());
}

//  FIXME BS NOW: delete (done in on_update_individual_physics)
// pub fn on_update_individual_position(
//     update: On<SetPositionEvent<IndividualIndex>>,
//     mut index: ResMut<super::World>,
// ) {
//     let i = update.0;
//     let individual = &update.1;

//     let position1 = &update.2;
//     let position2 = &update.1;

//     index.remove_individual(i, *position1);
//     index.insert_individual(i, individual.clone());
// }

// TODO: should be automatized (macro? derive ?)
pub fn on_forgotten_region(region: On<ForgottenRegion>, mut index: ResMut<super::World>) {
    tracing::trace!(name = "world-individual-forgotten-region", region=?region.0);
    index.remove_individuals(region.0);
}

pub fn on_update_individual_physics(
    update: On<UpdateIndividualPhysicsEvent>,
    mut index: ResMut<super::World>,
) {
    let (i, update) = (update.0, &update.1);
    let Some(mut individual) = index.get_individual(i).cloned() else {
        return;
    };

    // Here, must update index with all used values by oc_physics::step
    match update {
        oc_physics::update::Update::SetTile(tile, _) => {
            let position = *individual.position();
            individual.set_tile(*tile);
            index.remove_individual(i, position);
            index.insert_individual(i, individual);
        }
        oc_physics::update::Update::SetVolume(volume, _) => {
            let position = *individual.position();
            individual.set_volume(volume.clone());
            index.remove_individual(i, position);
            index.insert_individual(i, individual);
        }
        oc_physics::update::Update::SetPosition(_, _)
        | oc_physics::update::Update::SetRegion(_, _)
        | oc_physics::update::Update::PushForce(_)
        | oc_physics::update::Update::RemoveForce(_) => {}
    }
}
