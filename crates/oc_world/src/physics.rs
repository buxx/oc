// use oc_individual::{Individual, IndividualIndex, Update};
// use oc_physics::Event;
// use oc_projectile::{Projectile, ProjectileId};
// use oc_root::tile::Tile;

// use crate::update::WorldUpdate;

// pub trait Reactive<I, T: Clone + std::fmt::Debug> {
//     fn react(&self, i: I, event: &Event<T>) -> (Vec<Update>, Vec<WorldUpdate>);
// }

// impl Reactive<IndividualIndex, Tile> for Individual {
//     fn react(&self, _i: IndividualIndex, _event: &Event<Tile>) -> (Vec<Update>, Vec<WorldUpdate>) {
//         (vec![], vec![])
//     }
// }

// impl Reactive<ProjectileId, Tile> for Projectile {
//     fn react(&self, i: ProjectileId, event: &Event<Tile>) -> (Vec<Update>, Vec<WorldUpdate>) {
//         match event {
//             Event::NoTile | Event::Collision(_) => (vec![], vec![WorldUpdate::RemoveProjectile(i)]),
//         }
//     }
// }
