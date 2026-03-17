use bevy::prelude::*;
use oc_geo::{region::WorldRegionIndex, tile::TileXy};
use oc_physics::{
    Corps, Laws,
    collision::Material_,
    update::bevy::{Forces, Position, Region},
};
use oc_root::tile::Tile;
use oc_utils::d2::Xy;

use crate::world::tile::Tiles;

#[derive(Debug, Clone, Event)]
pub struct PhysicEvent<I>(I, oc_physics::Event<Tile>);

pub fn physics_step<I: Clone + Send + Sync + 'static, C: Component + AsRef<I>>(
    mut commands: Commands,
    time: Res<Time>,
    query: Query<(
        &C,
        &mut Position,
        &Region,
        &mut Forces,
        &Material_,
        &mut Transform,
    )>,
    tiles: Res<Tiles>,
) {
    tracing::trace!(name = "projectile-physics-start");
    let laws = Laws::default().tick_coeff(time.delta_secs() / 1.);

    for (object, mut position, region, mut forces, material, mut transform) in query {
        // TODO: Maybe performant bottleneck ?
        let tiles = |xy: Xy| {
            // NOTE: We must use the given tile xy and not the component position because closure position is the real position (computed by physics).
            let region: WorldRegionIndex = TileXy(xy).into();
            tiles
                .get(&region)
                .and_then(|tiles| tiles.get(&TileXy(xy).into()))
        };

        // let on_physics_event = |e| object.on_physics_event(e);
        let corps = Corps::new(&position.0, &forces.0, material.0); //, on_physics_event);
        let (position_, forces_, events) = oc_physics::step(&laws, &corps, tiles);

        position.0 = position_;
        forces.0 = forces_;
        transform.translation.x = position.0[0];
        transform.translation.y = position.0[1];

        for event in events {
            // Add observer for ProjectileId
            commands.trigger(PhysicEvent::<I>(object.as_ref().clone(), event))
        }
    }
}
