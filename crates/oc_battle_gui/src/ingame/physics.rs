use bevy::prelude::*;
use oc_geo::tile::TileXy;
use oc_physics::{
    Corps, Laws,
    collision::Material_,
    update::bevy::{Forces, Position, Region},
};
use oc_utils::d2::Xy;

use crate::world::tile::Tiles;

pub fn physics_step<C: Component>(
    time: Res<Time>,
    query: Query<
        (
            &mut Position,
            &Region,
            &mut Forces,
            &Material_,
            &mut Transform,
        ),
        With<C>,
    >,
    tiles: Res<Tiles>,
) {
    tracing::trace!(name = "projectile-physics-start");
    let laws = Laws::default().tick_coeff(time.delta_secs() / 1.);

    for (mut position, region, mut forces, material, mut transform) in query {
        // TODO: Maybe performant bottleneck ?
        let tiles = |xy: Xy| {
            tiles
                .get(&region.0.into())
                .and_then(|tiles| tiles.get(&TileXy(xy).into()))
        };
        let corps = Corps::new(&position.0, &forces.0, material.0);
        let (position_, forces_) = oc_physics::step(&laws, &corps, tiles);

        position.0 = position_;
        forces.0 = forces_;
        transform.translation.x = position.0[0];
        transform.translation.y = position.0[1];
    }
}
