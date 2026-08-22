use bevy::prelude::*;
use oc_geo::tile::{TileXy, WorldTileIndex};
use oc_individual::IndividualIndex;
use oc_physics::{
    IgnoreSide,
    collision::Material_,
    corps::Corps,
    update::bevy::{Forces, Position, Volumes},
};
use oc_projectile::ProjectileId;
use oc_root::{WcfgFrom, geo::ScreenVec2};
use oc_utils::{d2::Xy, let_some};

use crate::{ingame::projectile::ForgotProjectile, states::GameConfig, world::World};

#[derive(Debug, Clone, Event)]
pub struct PhysicEvent(oc_physics::Event<ObjectId>);

#[derive(Debug, Deref, Component)]
pub struct Direction(pub oc_utils::d2::Direction);

pub fn physics_step<I, C>(
    mut commands: Commands,
    g: Res<GameConfig>,
    time: Res<Time>,
    query: Query<(
        &C,
        &mut Position,
        &mut Forces,
        &Material_,
        &Volumes,
        &Direction,
        &mut Transform,
    )>,
    index: Res<World>,
) where
    I: Clone + Send + Sync + Into<ObjectId> + std::fmt::Debug + 'static,
    C: Component + AsRef<I>,
{
    let_some!(g = &g.0, return);

    // tracing::trace!(name = "projectile-physics-start");
    let delta = time.delta_secs() / 1.;

    for (object, mut position, mut forces, material, volume, direction, mut transform) in query {
        let i = object.as_ref();
        tracing::trace!(name = "projectile-physics-object", i=?i);

        // TODO: Maybe performant bottleneck ?
        let objects = |xy: Xy| {
            // NOTE: We must use the given tile xy and not the component position because it is the real position (computed by physics just now).
            // let region: WorldRegionIndex = TileXy(xy).into();
            index.at(&g.w, TileXy(xy))
        };

        // TODO: test perf with references in Corps
        let corps = Corps::new(
            i.clone(),
            position.0,
            direction.0,
            forces.0.clone(),
            material.0,
            volume.0.clone(),
            None,
            IgnoreSide::All, // Gui prevent all "side" collision (let server manage it)
        );
        let (position_, forces_, events) = oc_physics::step(
            &g.w,
            &g.mod_,
            delta,
            (i.clone(), &corps),
            objects,
            g.w.ignore_firsts_physics_pixels as usize,
            "gui",
        );

        position.0 = position_;
        forces.0 = forces_;
        let position__ = ScreenVec2::from_(position_, &g.w);
        transform.translation.x = position__.x;
        transform.translation.y = position__.y;

        for event in events {
            commands.trigger(PhysicEvent(event))
        }
    }
}

pub fn on_physics_event(event: On<PhysicEvent>, mut commands: Commands) {
    match &event.0 {
        oc_physics::Event::NoTile(id) => match id {
            ObjectId::Individual(_) | ObjectId::Tile(_) => {}
            ObjectId::Projectile(i) => {
                commands.trigger(ForgotProjectile(*i));
            }
        },
        // TODO: implement fragments / rebound
        oc_physics::Event::Collision(a, b) => {
            match (a, b) {
                (ObjectId::Individual(_), ObjectId::Individual(_))
                | (ObjectId::Individual(_), ObjectId::Projectile(_))
                | (ObjectId::Projectile(_), ObjectId::Projectile(_))
                | (ObjectId::Individual(_), ObjectId::Tile(_))
                | (ObjectId::Tile(_), ObjectId::Individual(_))
                | (ObjectId::Tile(_), ObjectId::Projectile(_))
                | (ObjectId::Tile(_), ObjectId::Tile(_)) => {}
                (ObjectId::Projectile(_), ObjectId::Individual(_)) => {
                    // TODO: compute kill too ? or wait server
                }
                (ObjectId::Projectile(i), ObjectId::Tile(_)) => {
                    // TODO impact sound
                    commands.trigger(ForgotProjectile(*i));
                }
            }
        }
    }
}

// TODO: move code (use same than server, refacto it)
#[derive(Debug, Clone, serde::Serialize, PartialEq)]
pub enum ObjectId {
    #[allow(unused)]
    Individual(IndividualIndex),
    Projectile(ProjectileId),
    #[allow(unused)]
    Tile(WorldTileIndex),
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
