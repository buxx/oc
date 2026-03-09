use bevy::color::palettes::css::RED;
use bevy::prelude::*;
use oc_geo::region::{Region as _, WorldRegionIndex};
use oc_physics::Physic;
use oc_physics::collision::{Material_, Materials};
use oc_physics::update::bevy::{Forces, PhysicsPlugin, Position, Region, Tile};
use oc_utils::bevy::EntityMapping;

use crate::entity::projectile::ProjectileId;
use crate::ingame;
use crate::ingame::draw::Z_INDIVIDUAL;
use crate::ingame::input::individual::UpdateProjectilePhysicsEvent;
use crate::ingame::input::projectile::InsertProjectileEvent;
use crate::ingame::region::ForgottenRegion;
use crate::states::AppState;

pub fn on_insert_projectile(
    projectile: On<InsertProjectileEvent>,
    mut commands: Commands,
    mut state: ResMut<EntityMapping<oc_projectile::ProjectileId>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    tracing::trace!(name="spawn-projectile", i=?projectile.0, position=?projectile.1.position(), forces=?projectile.1.forces());
    let entity = commands
        .spawn((
            ProjectileId(projectile.0),
            Position(projectile.1.position().clone()),
            Tile(projectile.1.tile().clone()),
            Region(projectile.1.region().clone()),
            Forces(projectile.1.forces().clone()),
            Material_(Materials::Traversable),
            Mesh2d(meshes.add(Circle::new(2.5))),
            MeshMaterial2d(materials.add(Color::from(RED))),
            Transform::from_xyz(
                projectile.1.position()[0],
                projectile.1.position()[1],
                Z_INDIVIDUAL,
            ),
        ))
        .id();
    state.insert(projectile.0, entity);
}

pub struct ProjectilePlugin;

impl Plugin for ProjectilePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PhysicsPlugin::<
            oc_projectile::ProjectileId,
            UpdateProjectilePhysicsEvent,
        >::default())
            .add_observer(on_insert_projectile)
            .init_resource::<EntityMapping<oc_projectile::ProjectileId>>()
            .add_observer(on_forgotten_region)
            .add_systems(
                Update,
                ingame::physics::physics_step::<ProjectileId>.run_if(in_state(AppState::InGame)),
            );
    }
}

// TODO: should be automatized (macro? derive ?)
fn on_forgotten_region(
    region: On<ForgottenRegion>,
    mut commands: Commands,
    mut state: ResMut<EntityMapping<oc_projectile::ProjectileId>>,
    query: Query<(Entity, &Region, &ProjectileId)>,
) {
    for (entity, region_, projectile) in query {
        let region_: WorldRegionIndex = region_.0.into();
        if region_ == region.0 {
            tracing::trace!(name = "remove-projectile", i=?projectile);
            commands.entity(entity).despawn();
            state.remove(&projectile.0);
        }
    }
}
