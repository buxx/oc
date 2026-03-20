use bevy::prelude::*;
use oc_geo::tile::TileXy;
use oc_individual::IndividualIndex;
use oc_physics::{
    Corps, Laws,
    collision::Material_,
    update::bevy::{Forces, Position, Volume},
};
use oc_projectile::ProjectileId;
use oc_utils::{bevy::EntityMapping, d2::Xy};

use crate::world::World;

#[derive(Debug, Clone, Event)]
pub struct PhysicEvent<I: Clone + std::fmt::Debug>(I, oc_physics::Event<I, ObjectId>);

pub fn physics_step<I: Clone + Send + Sync + std::fmt::Debug + 'static, C: Component + AsRef<I>>(
    mut commands: Commands,
    time: Res<Time>,
    query: Query<(
        &C,
        &mut Position,
        &mut Forces,
        &Material_,
        &Volume,
        &mut Transform,
    )>,
    index: Res<World>,
) {
    tracing::trace!(name = "projectile-physics-start");
    let laws = Laws::default().tick_coeff(time.delta_secs() / 1.);

    for (object, mut position, mut forces, material, volume, mut transform) in query {
        let i = object.as_ref();
        tracing::trace!(name = "projectile-physics-object", i=?i);

        // TODO: Maybe performant bottleneck ?
        let objects = |xy: Xy| {
            // NOTE: We must use the given tile xy and not the component position because it is the real position (computed by physics just now).
            // let region: WorldRegionIndex = TileXy(xy).into();
            index.at(TileXy(xy))
        };

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
    // Projectile(ProjectileId),
    // Tile(WorldTileIndex),
}
