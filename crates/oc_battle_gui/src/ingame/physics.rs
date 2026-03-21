use bevy::prelude::*;
use oc_geo::tile::TileXy;
use oc_individual::IndividualIndex;
use oc_physics::{
    Corps, Laws,
    collision::Material_,
    update::bevy::{Forces, Position, Volume},
};
use oc_projectile::ProjectileId;
use oc_utils::d2::Xy;

use crate::{ingame::projectile::ForgotProjectile, world::World};

#[derive(Debug, Clone, Event)]
pub struct PhysicEvent(oc_physics::Event<ObjectId>);

pub fn physics_step<I, C>(
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
) where
    I: Clone + Send + Sync + Into<ObjectId> + std::fmt::Debug + 'static,
    C: Component + AsRef<I>,
{
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
            commands.trigger(PhysicEvent(event))
        }
    }
}

pub fn on_physics_event(event: On<PhysicEvent>, mut commands: Commands) {
    match &event.0 {
        oc_physics::Event::NoTile(id) => match id {
            ObjectId::Individual(_) => {}
            ObjectId::Projectile(i) => {
                commands.trigger(ForgotProjectile(*i));
            }
        },
        // FIXME: implement fragments / rebound
        oc_physics::Event::Collision(a, b) => {
            match (a, b) {
                (ObjectId::Individual(_), ObjectId::Individual(_)) => {}
                (ObjectId::Individual(_), ObjectId::Projectile(_)) => {}
                (ObjectId::Projectile(_projectile_id), ObjectId::Individual(_individual_index)) => {
                    // TODO: bam
                }
                (ObjectId::Projectile(_), ObjectId::Projectile(_)) => {}
            }
        }
    }
}

// TODO: move code (use same than server, refacto it)
#[derive(Debug, Clone)]
pub enum ObjectId {
    Individual(IndividualIndex),
    Projectile(ProjectileId),
    // Tile(WorldTileIndex),
}

impl From<IndividualIndex> for ObjectId {
    fn from(value: IndividualIndex) -> Self {
        ObjectId::Individual(value)
    }
}

impl From<ProjectileId> for ObjectId {
    fn from(value: ProjectileId) -> Self {
        ObjectId::Projectile(value)
    }
}
