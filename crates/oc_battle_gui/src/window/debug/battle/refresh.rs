use std::time::Instant;

use bevy::prelude::*;
use oc_physics::update::bevy::{Forces, Position, Region, Tile};

use crate::{
    entity::{individual::IndividualIndex, projectile::ProjectileId},
    ingame::camera::State,
    states,
    window::debug::{physics::PhysicsRepr, subject::Subject},
};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Refresh {
    EachFrame,
    #[default]
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
) {
    #[allow(irrefutable_let_patterns)] // TODO: no more irrefutable when more windows
    if let Some(crate::window::Window::BattleDebug(window)) = &mut window.0 {
        window.context.regions = camera.regions.clone().unwrap_or(vec![]);
        window.context.individuals = individuals
            .iter()
            .map(|(id, position, tile, region, forces)| {
                Subject::new(
                    id.0.clone(),
                    PhysicsRepr::new(
                        position.0.clone(),
                        tile.0.clone(),
                        region.0.clone(),
                        forces.0.clone(),
                    ),
                )
            })
            .collect();
        window.context.projectiles = projectiles
            .iter()
            .map(|(id, position, tile, region, forces)| {
                Subject::new(
                    id.0.clone(),
                    PhysicsRepr::new(
                        position.0.clone(),
                        tile.0.clone(),
                        region.0.clone(),
                        forces.0.clone(),
                    ),
                )
            })
            .collect();
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
