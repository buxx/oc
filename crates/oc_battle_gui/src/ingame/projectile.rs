use bevy::color::palettes::css::RED;
use bevy::prelude::*;
use oc_geo::region::{Region as _, WorldRegionIndex};
use oc_physics::Physic;
use oc_physics::collision::{Material_, Materials};
use oc_physics::update::bevy::{
    Forces, PhysicsPlugin, Position, Region, SetPositionEvent, Tile, Volume,
};
use oc_root::Wcfg;
use oc_root::y::Y;
use oc_utils::bevy::EntityMapping;

use crate::entity::projectile::ProjectileId;
use crate::ingame;
use crate::ingame::draw::Z_PROJECTILE;
use crate::ingame::input::individual::UpdateProjectilePhysicsEvent;
use crate::ingame::input::projectile::InsertProjectileEvent;
use crate::ingame::region::ForgottenRegion;
use crate::states::{AppState, Mod};

#[derive(Debug, Deref, Event)]
pub struct ForgotProjectile(pub oc_projectile::ProjectileId);

pub struct ProjectilePlugin;

impl Plugin for ProjectilePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PhysicsPlugin::<
            oc_projectile::ProjectileId,
            UpdateProjectilePhysicsEvent,
        >::default())
            .add_observer(on_insert_projectile)
            .init_resource::<EntityMapping<oc_projectile::ProjectileId>>()
            .add_observer(on_update_position)
            .add_observer(on_forgotten_region)
            .add_observer(on_forgot_projectile)
            .add_systems(
                Update,
                ingame::physics::physics_step::<oc_projectile::ProjectileId, ProjectileId>
                    .run_if(in_state(AppState::InGame)),
            );
    }
}

pub fn on_insert_projectile(
    projectile: On<InsertProjectileEvent>,
    mut commands: Commands,
    w: Res<Wcfg>,
    mod_: Res<Mod>,
    mut state: ResMut<EntityMapping<oc_projectile::ProjectileId>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let Some(w) = &w.0 else {
        return;
    };
    let Some(mod_) = &mod_.0 else {
        return;
    };
    tracing::trace!(name="spawn-projectile", i=?projectile.0, position=?projectile.1.position(), forces=?projectile.1.forces(w));

    let position = projectile.1.position();
    let line = Polyline2d::new(vec![Vec2::new(position[0], position[1].to_gui_y(w))]);
    let entity = commands
        .spawn((
            ProjectileId(projectile.0),
            Position(*position),
            Tile(projectile.1.tile()),
            Region(projectile.1.region()),
            Forces(projectile.1.forces(w).clone()),
            Material_(Materials::Traversable),
            Volume(projectile.1.volume(*position, w, mod_).clone()),
            Mesh2d(meshes.add(line)),
            MeshMaterial2d(materials.add(Color::from(RED))),
            Transform::from_xyz(
                projectile.1.position()[0],
                projectile.1.position()[1].to_gui_y(w),
                Z_PROJECTILE,
            ),
        ))
        .id();
    state.insert(projectile.0, entity);
}

fn on_update_position(
    position: On<SetPositionEvent<oc_projectile::ProjectileId>>,
    projectiles: Res<EntityMapping<oc_projectile::ProjectileId>>,
    mut meshes: ResMut<Assets<Mesh>>,
    query: Query<&Mesh2d>,
) {
    let (i, position, previous) = (position.0, &position.1, &position.2);
    let Some(entity) = projectiles.get(&i) else {
        return;
    };
    let Ok(mesh) = query.get(*entity) else {
        return;
    };
    let Some(mesh) = meshes.get_mut(mesh) else {
        return;
    };

    let position = Vec2::new(position[0], position[1]);
    let previous = Vec2::new(previous[0], previous[1]);
    let relative = previous - position;
    let relative = Vec2::new(relative.x, -relative.y);
    *mesh = Polyline2d::new(vec![Vec2::new(0.0, 0.0), relative]).into();
}

// TODO: should be automatized (macro? derive ?)
fn on_forgotten_region(
    region: On<ForgottenRegion>,
    mut commands: Commands,
    query: Query<(&Region, &ProjectileId)>,
) {
    for (region_, i) in query {
        let region_: WorldRegionIndex = region_.0;
        if region_ == region.0 {
            tracing::trace!(name = "trigger-forgot-projectile", i=?i);
            commands.trigger(ForgotProjectile(i.0));
        }
    }
}

pub fn on_forgot_projectile(
    projectile: On<ForgotProjectile>,
    mut commands: Commands,
    mut projectiles: ResMut<EntityMapping<oc_projectile::ProjectileId>>,
) {
    if let Some(entity) = projectiles.remove(&projectile.0) {
        tracing::trace!(name = "remove-projectile", i=?projectile.0);
        commands.entity(entity).despawn();
    }
}
