use bevy::color::palettes::css::RED;
use bevy::prelude::*;
use oc_geo::region::{Region as _, RegionXy, WorldRegionIndex};
use oc_geo::tile::TileXy;
use oc_physics::collision::Materials;
use oc_physics::{Force, Laws, Physic};

use crate::entity::geo::Position;
use crate::entity::physics::Forces;
use crate::entity::projectile::ProjectileId;
use crate::entity::world::Tile;
use crate::entity::world::region::Region;
use crate::ingame::draw::Z_INDIVIDUAL;
use crate::ingame::input::projectile::{InsertProjectileEvent, UpdateProjectileEvent};
use crate::ingame::region::ForgottenRegion;
use crate::ingame::state::State;
use crate::states::AppState;

// FIXME: refactor accoridng to projectile similar code

#[derive(Debug, Event)]
pub struct UpdatePositionEvent(oc_projectile::ProjectileId, [f32; 2]);

#[derive(Debug, Event)]
pub struct UpdateTileEvent(oc_projectile::ProjectileId, TileXy);

#[derive(Debug, Event)]
pub struct UpdateRegionEvent(oc_projectile::ProjectileId, RegionXy);

#[derive(Debug, Event)]
pub struct PushForceEvent(oc_projectile::ProjectileId, Force);

#[derive(Debug, Event)]
pub struct RemoveForceEvent(oc_projectile::ProjectileId, Force);

pub fn on_insert_projectile(
    projectile: On<InsertProjectileEvent>,
    mut commands: Commands,
    mut state: ResMut<State>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    tracing::trace!(name="spawn-projectile", i=?projectile.0, position=?projectile.1.position());
    let entity = commands
        .spawn((
            ProjectileId(projectile.0),
            Position(projectile.1.position().clone()),
            Tile(projectile.1.tile().clone()),
            Region(projectile.1.region().clone()),
            Forces(projectile.1.forces().clone()),
            Mesh2d(meshes.add(Circle::new(2.5))),
            MeshMaterial2d(materials.add(Color::from(RED))),
            Transform::from_xyz(
                projectile.1.position()[0],
                projectile.1.position()[1],
                Z_INDIVIDUAL,
            ),
        ))
        .id();
    state.projectiles.insert(projectile.0, entity);
}

pub fn on_update_projectile(update: On<UpdateProjectileEvent>, mut commands: Commands) {
    let (i, update) = (update.0, &update.1);

    // TODO: use macro to automatise events declaration and mapping here
    match update {
        oc_projectile::Update::Physics(update) => match update {
            oc_physics::update::Update::UpdatePosition(position) => {
                commands.trigger(UpdatePositionEvent(i, *position));
            }
            oc_physics::update::Update::UpdateTile(tile) => {
                commands.trigger(UpdateTileEvent(i, *tile));
            }
            oc_physics::update::Update::UpdateRegion(region) => {
                commands.trigger(UpdateRegionEvent(i, *region));
            }
            oc_physics::update::Update::PushForce(force) => {
                commands.trigger(PushForceEvent(i, force.clone()));
            }
            oc_physics::update::Update::RemoveForce(force) => {
                commands.trigger(RemoveForceEvent(i, force.clone()));
            }
        },
    }
}

pub struct ProjectilePlugin;

impl Plugin for ProjectilePlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_insert_projectile)
            .add_observer(on_update_projectile)
            .add_observer(on_update_position_event)
            .add_observer(on_update_tile_event)
            .add_observer(on_update_region_event)
            .add_observer(on_push_force_event)
            .add_observer(on_remove_force_event)
            .add_observer(on_forgotten_region)
            .add_systems(Update, physics.run_if(in_state(AppState::InGame)));
    }
}

fn physics(
    time: Res<Time>,
    query: Query<(&mut Position, &mut Forces, &mut Transform), With<ProjectileId>>,
) {
    let laws = Laws::default().tick_coeff(time.delta_secs() / 1.);
    let tiles = |_| Some(&oc_world::tile::Tile::ShortGrass); // FIXME

    for (mut position, mut forces, mut transform) in query {
        let projectile = Projectile {
            position: &position.0,
            forces: &forces.0,
        };
        let (position_, forces_) = oc_physics::step(&laws, &projectile, tiles);
        position.0 = position_;
        forces.0 = forces_;
        transform.translation.x = position.0[0];
        transform.translation.y = position.0[1];
    }
}

pub struct Projectile<'a> {
    position: &'a [f32; 2],
    forces: &'a Vec<Force>,
}

impl<'a> Physic for Projectile<'a> {
    fn position(&self) -> &[f32; 2] {
        &self.position
    }

    fn forces(&self) -> &Vec<Force> {
        &self.forces
    }
}

impl<'a> oc_physics::collision::Material for Projectile<'a> {
    fn material(&self) -> Materials {
        Materials::Traversable
    }
}

fn on_update_position_event(
    position: On<UpdatePositionEvent>,
    mut query: Query<(&mut Position, &mut Transform)>,
    state: Res<State>,
) {
    let Some(entity) = state.projectiles.get(&position.0) else {
        return;
    };
    let Ok((mut position_, mut transform)) = query.get_mut(*entity) else {
        return;
    };
    tracing::trace!(name = "update-projectile-position", i=?position.0.0, position=?position.1);

    position_.0 = position.1;
    transform.translation = Vec3::new(position.1[0], position.1[1], Z_INDIVIDUAL);
}

fn on_update_tile_event(tile: On<UpdateTileEvent>, mut query: Query<&mut Tile>, state: Res<State>) {
    let Some(entity) = state.projectiles.get(&tile.0) else {
        return;
    };
    let Ok(mut tile_) = query.get_mut(*entity) else {
        return;
    };
    tracing::trace!(name="update-projectile-tile", i=?tile.0, tile=?tile.1);

    tile_.0 = tile.1;
}

fn on_update_region_event(
    region: On<UpdateRegionEvent>,
    mut query: Query<&mut Region>,
    state: Res<State>,
) {
    let Some(entity) = state.projectiles.get(&region.0) else {
        return;
    };
    let Ok(mut region_) = query.get_mut(*entity) else {
        return;
    };
    tracing::trace!(name="update-projectile-region", i=?region.0, region=?region.1);

    region_.0 = region.1;
}

fn on_push_force_event(
    force: On<PushForceEvent>,
    mut query: Query<&mut Forces>,
    state: Res<State>,
) {
    let Some(entity) = state.projectiles.get(&force.0) else {
        return;
    };
    let Ok(mut forces) = query.get_mut(*entity) else {
        return;
    };
    tracing::trace!(name = "update-projectile-force-push", i=?force.0, force=?force.1);

    forces.0.push(force.1.clone());
}

fn on_remove_force_event(
    force: On<RemoveForceEvent>,
    mut query: Query<&mut Forces>,
    state: Res<State>,
) {
    let Some(entity) = state.projectiles.get(&force.0) else {
        return;
    };
    let Ok(mut forces) = query.get_mut(*entity) else {
        return;
    };
    tracing::trace!(name = "update-projectile-force-remove", i=?force.0, force=?force.1);

    forces.0.retain(|f| f != &force.1);
}

fn on_forgotten_region(
    region: On<ForgottenRegion>,
    mut commands: Commands,
    mut state: ResMut<State>,
    query: Query<(Entity, &Region, &ProjectileId)>,
) {
    for (entity, region_, projectile) in query {
        let region_: WorldRegionIndex = region_.0.into();
        if region_ == region.0 {
            tracing::trace!(name = "remove-projectile", i=?projectile);
            commands.entity(entity).despawn();
            state.projectiles.remove(&projectile.0);
        }
    }
}
