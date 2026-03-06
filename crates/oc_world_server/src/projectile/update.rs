// use std::sync::{Arc, mpsc::Sender};

// use message_io::network::Endpoint;
// use oc_geo::{UpdateGeo, region::Region};
// use oc_network::ToClient;
// use oc_physics::UpdatePhysic;
// use oc_projectile::{ProjectileId, Update, network};

// use crate::{routing::Listening, state::State};

// // TODO: lot of common things with individual, refactor !
// pub fn write(
//     update: Update,
//     id: ProjectileId,
//     state: &Arc<State>,
//     output: &Sender<(Endpoint, ToClient)>,
// ) {
//     let (region_before, region_after, id) = {
//         let mut world = state.world_mut();
//         let mut indexes = state.indexes_mut();
//         let Some(projectile) = world.projectile_mut(&id) else {
//             return; // TODO: its an possible or we should panic ?
//         };
//         let region_before = projectile.region().clone();

//         match update.clone() {
//             Update::UpdatePosition(position) => {
//                 projectile.set_position(position);
//             }
//             Update::UpdateTile(tile) => {
//                 let before = projectile.tile().clone();
//                 projectile.set_tile(tile);
//                 indexes.update_projectile_tile(id, before, tile);
//             }
//             Update::UpdateRegion(region) => {
//                 let before = projectile.region().clone();
//                 projectile.set_region(region);
//                 indexes.update_projectile_region(id, before, region);
//             }
//             Update::PushForce(force) => {
//                 projectile.push_force(force);
//             }
//             Update::RemoveForce(force) => {
//                 projectile.remove_force(&force);
//             }
//         }

//         (region_before.clone(), projectile.region().clone(), id)
//     };

//     tracing::trace!(name="projectile-update-write-broadast-update", i=?id, update=?update);
//     broadcast(
//         state,
//         Listening::Regions(vec![region_before.clone().into()]),
//         vec![network::Projectile::Update(id, update)],
//         output,
//     );

//     if region_before != region_after {
//         let world = state.world();
//         let Some(projectile) = world.projectile(&id) else {
//             return;
//         };

//         tracing::trace!(name="projectile-update-write-broadast-insert", i=?id, id=?id);
//         broadcast(
//             state,
//             Listening::Regions(vec![]),
//             vec![network::Projectile::Insert(id, projectile.clone())],
//             output,
//         );
//     }
// }

// fn broadcast(
//     state: &Arc<State>,
//     filter: Listening,
//     messages: Vec<network::Projectile>,
//     output: &Sender<(Endpoint, ToClient)>,
// ) {
//     state
//         .listeners()
//         .find(filter)
//         .into_iter()
//         .for_each(|endpoint| {
//             messages
//                 .iter()
//                 .for_each(|message| output.send((endpoint, message.clone().into())).unwrap()) // TODO
//         });
// }
