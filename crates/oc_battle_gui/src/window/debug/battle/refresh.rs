use std::time::Instant;

use bevy::prelude::*;
use oc_geo::{region::RegionXy, tile::TileXy};
use oc_physics::update::bevy::{Forces, Position, Region, Tile};
use oc_root::y::Y;

use crate::world::World;
use crate::{
    entity::{individual::IndividualIndex, projectile::ProjectileId},
    ingame::camera::State,
    states,
    window::debug::{physics::PhysicsRepr, subject::Subject},
};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Refresh {
    #[default]
    EachFrame,
    X100ms,
    X1s,
}

impl Refresh {
    fn as_millis(&self) -> u128 {
        match self {
            Refresh::EachFrame => 0,
            Refresh::X100ms => 100,
            Refresh::X1s => 1000,
        }
    }
}

pub fn on_refresh(
    _: On<super::Refresh>,
    mut window: ResMut<states::Window>,
    camera: Res<State>,
    individuals: Query<(&IndividualIndex, &Position, &Tile, &Region, &Forces)>,
    projectiles: Query<(&ProjectileId, &Position, &Tile, &Region, &Forces)>,
    window_: Single<&bevy::window::Window>,
    camera_: Single<(&Camera, &GlobalTransform)>,
    world: Res<World>,
) {
    #[allow(irrefutable_let_patterns)] // TODO: no more irrefutable when more windows
    if let Some(crate::window::Window::BattleDebug(window)) = &mut window.0 {
        window.context.regions = camera.regions.clone().unwrap_or(vec![]);
        window.context.individuals = individuals
            .iter()
            .map(|(id, position, _, region, _)| {
                Subject::new(
                    id.0.clone(),
                    PhysicsRepr::new(
                        position.0.clone(),
                        // tile.0.clone(),
                        region.0.clone(),
                        // forces.0.clone(),
                    ),
                )
            })
            .collect();
        window.context.projectiles = projectiles
            .iter()
            .map(|(id, position, _, region, _)| {
                Subject::new(
                    id.0.clone(),
                    PhysicsRepr::new(
                        position.0.clone(),
                        // tile.0.clone(),
                        region.0.clone(),
                        // forces.0.clone(),
                    ),
                )
            })
            .collect();

        let (camera, transform) = *camera_;
        if let Some(cursor) = window_.cursor_position() {
            if let Ok(worldp) = camera.viewport_to_world_2d(transform, cursor) {
                let point = Vec2::new(worldp.x, worldp.y.to_gui_y());
                let tile: TileXy = [point.x, point.y].into();
                let tile_ = world.tile(tile);
                let tile_ = tile_
                    .map(|t| format!("{} ({})", t.nature, t.z))
                    .unwrap_or_default();
                let region: RegionXy = tile.into();

                window.context.cursor = Some(cursor);
                window.context.point = Some(point);
                window.context.tile = Some((tile, tile_));
                window.context.region = Some(region);
            };
        };

        window.last = Instant::now();
    }
}

pub fn trigger_refresh(window: Res<states::Window>, mut commands: Commands) {
    #[allow(irrefutable_let_patterns)] // TODO: no more irrefutable when more windows
    if let Some(crate::window::Window::BattleDebug(window)) = &window.0 {
        if window.last.elapsed().as_millis() > window.context.refresh.as_millis() {
            commands.trigger(super::Refresh);
        }
    }
}
