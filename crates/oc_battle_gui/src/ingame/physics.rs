use bevy::prelude::*;
use oc_geo::tile::{TileXy, WorldTileIndex};
use oc_individual::IndividualIndex;
use oc_physics::{
    Corps, Laws, Physic,
    collision::{Material, Material_, Materials},
    update::bevy::{Forces, Position, Region, Volume},
};
use oc_projectile::ProjectileId;
use oc_root::{GEO_PIXELS_PER_TILE, tile::Tile};
use oc_utils::{bevy::EntityMapping, d2::Xy};

use crate::{entity, world::World};

#[derive(Debug, Clone, Event)]
pub struct PhysicEvent<I: Clone + std::fmt::Debug>(I, oc_physics::Event<I, ObjectId>);

pub fn physics_step<I: Clone + Send + Sync + std::fmt::Debug + 'static, C: Component + AsRef<I>>(
    mut commands: Commands,
    time: Res<Time>,
    query: Query<(
        &C,
        &mut Position,
        &Region,
        &mut Forces,
        &Material_,
        &Volume,
        &mut Transform,
    )>,
    // individuals_index: Res<Individuals>,
    // individuals_entity: Res<EntityMapping<IndividualIndex>>,
    // individuals_query: Query<
    //     (&Position, &Material_, &Volume),
    //     With<entity::individual::IndividualIndex>,
    // >,
    // tiles_index: Res<Tiles>,
    index: Res<World>,
) {
    tracing::trace!(name = "projectile-physics-start");
    let laws = Laws::default().tick_coeff(time.delta_secs() / 1.);

    for (object, mut position, region, mut forces, material, volume, mut transform) in query {
        let i = object.as_ref();
        tracing::trace!(name = "projectile-physics-object", i=?i);

        // TODO: Maybe performant bottleneck ?
        let objects = |xy: Xy| {
            // NOTE: We must use the given tile xy and not the component position because it is the real position (computed by physics just now).
            // let region: WorldRegionIndex = TileXy(xy).into();
            index.at(TileXy(xy))

            // let individuals = individuals_index.at(&xy);
            // let mut objects: Vec<(ObjectId, [f32; 2], Materials, oc_physics::volume::Volume)> =
            //     individuals
            //         .iter()
            //         .filter_map(|i| {
            //             if let Some(individual) = individuals_entity.get(i) {
            //                 let Ok((position, material, volume)) =
            //                     individuals_query.get(*individual)
            //                 else {
            //                     return None;
            //                 };
            //                 Some((
            //                     ObjectId::Individual(*i),
            //                     position.0,
            //                     material.0,
            //                     volume.0.clone(),
            //                 ))
            //             } else {
            //                 None
            //             }
            //         })
            //         .collect();

            // // FIXME BS NOW: need z axis to consider tile volume ...
            // if let Some(_tile) = tiles_index.at(xy) {
            //     let tile = TileXy(xy);
            //     let i: WorldTileIndex = tile.into();
            //     let position: [f32; 2] = tile.into();
            //     // TODO: materials according to tile type ? (and for z ?)
            //     objects.push((
            //         ObjectId::Tile(i),
            //         position,
            //         Materials::Traversable, // FIXMEBS NOW: z
            //         oc_physics::volume::Volume::Square2d {
            //             width: GEO_PIXELS_PER_TILE as f32,
            //             height: GEO_PIXELS_PER_TILE as f32,
            //         },
            //     ));
            // }

            // objects
        };

        // let on_physics_event = |e| object.on_physics_event(e);

        // let world = World {
        //     individuals_index: &individuals_index,
        //     individuals_entity: &individuals_entity,
        //     individuals_query: &individuals_query,
        //     tiles_index: &tiles_index,
        // };
        // FIXME: test perf with references in Corps
        let corps = Corps::new(
            i.clone(),
            position.0,
            forces.0.clone(),
            material.0,
            volume.0.clone(),
        ); //, on_physics_event);
        let (position_, forces_, events) = oc_physics::step(&laws, (i.clone(), &corps), &objects);

        position.0 = position_;
        forces.0 = forces_;
        transform.translation.x = position.0[0];
        transform.translation.y = position.0[1];

        for event in events {
            // Add observer for ProjectileId
            commands.trigger(PhysicEvent::<I>(i.clone(), event))
        }
    }
}

pub fn on_projectile_physics_event(
    event: On<PhysicEvent<ProjectileId>>,
    mut commands: Commands,
    mut projectiles: ResMut<EntityMapping<ProjectileId>>,
) {
    match &event.1 {
        oc_physics::Event::NoTile => {
            if let Some(entity) = projectiles.remove(&event.0) {
                commands.entity(entity).despawn();
            }
        }
        oc_physics::Event::Collision(_) => {
            // FIXME BS NOW: implement fragments / rebound
            // Code must be refactored to be used by GUI for instant display
            // and by server to game logic
            if let Some(entity) = projectiles.remove(&event.0) {
                commands.entity(entity).despawn();
            }
        }
    }
}

// TODO: move code (use same than server, refacto it)
#[derive(Debug, Clone)]
pub enum ObjectId {
    Individual(IndividualIndex),
    Projectile(ProjectileId),
    Tile(WorldTileIndex),
}

// // // TODO: move code
// #[derive(Debug)]
// pub enum Object {
//     Individual(Corps<IndividualIndex>),
//     Projectile(Corps<ProjectileId>),
//     Tile(TileXy, Tile),
// }

// impl<'a> Physic for Object<'a> {
//     fn position(&self) -> &[f32; 2] {
//         todo!()
//     }

//     fn forces(&self) -> &Vec<oc_physics::Force> {
//         todo!()
//     }

//     fn volume(&self) -> &oc_physics::volume::Volume {
//         todo!()
//     }
// }

// impl<'a> Material for Object<'a> {
//     fn material(&self) -> oc_physics::collision::Materials {
//         todo!()
//     }
// }
