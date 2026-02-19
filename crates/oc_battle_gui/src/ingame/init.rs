use bevy::prelude::*;
use oc_geo::tile::TileXy;
use oc_network::ToServer;
use oc_utils::d2::Xy;

use crate::network::output::ToServerEvent;

pub fn init(mut commands: Commands) {
    let listen: ToServerEvent = ToServer::Listen(TileXy(Xy(0, 0)), TileXy(Xy(10, 10))).into();
    let refresh: ToServerEvent = ToServer::Refresh.into();

    commands.trigger(listen);
    commands.trigger(refresh);
}
