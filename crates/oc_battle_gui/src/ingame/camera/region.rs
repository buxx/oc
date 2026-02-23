use bevy::prelude::*;
use oc_geo::{
    region::{RegionXy, WorldRegionIndex},
    tile::TileXy,
};
use oc_network::ToServer;
use oc_utils::d2::Xy;

use crate::{
    ingame::region::{ForgottenRegion, ListeningRegion},
    network::output::ToServerEvent,
};

pub const REGIONS_WIDTH: u64 = 11;
pub const REGIONS_HEIGHT: u64 = 11;

#[derive(Debug, Clone)]
pub struct Region(pub WorldRegionIndex, pub bool);

// FIXME BS NOW: despawn des invidivuals quand arrêt de suivis d'une region
// FIXME BS NOW: quand individual change de region, il faut l'envoyer en entier au clients potentiels (qu'a ceux qui n'ecoutait pas la region de provenance ?)
pub fn update_regions(mut commands: Commands, mut state: ResMut<super::State>) {
    let Some(center) = state.center else { return };
    static EMPTY: Vec<Region> = vec![];
    let current = state.regions.as_ref().unwrap_or(&EMPTY);
    let regions = regions(center);

    let new: Vec<Region> = regions
        .iter()
        .filter(|region| !current.iter().find(|r| r.0 == **region).is_some())
        .map(|i| Region(*i, false))
        .collect();

    let still: Vec<Region> = current
        .iter()
        .filter_map(|region| {
            regions
                .iter()
                .find(|i| **i == region.0)
                .map(|_| region.clone())
        })
        .collect();

    let old: Vec<WorldRegionIndex> = current
        .iter()
        .filter(|region| !regions.iter().find(|i| **i == region.0).is_some())
        .map(|r| r.0)
        .collect();

    for new_ in &new {
        commands.trigger(ToServerEvent(ToServer::ListenRegion(new_.0)));
        commands.trigger(ListeningRegion(new_.0));
    }

    for old_ in &old {
        commands.trigger(ToServerEvent(ToServer::ForgotRegion(*old_)));
        commands.trigger(ForgottenRegion(*old_));
    }

    let now = vec![new, still].concat();
    state.regions = Some(now);
}

pub fn regions(center: Vec2) -> Vec<WorldRegionIndex> {
    let center: TileXy = [center.x, center.y].into();
    let center: RegionXy = center.into();

    let from_x_minus = REGIONS_WIDTH / 2;
    let from_x = center.0.0 - from_x_minus.min(center.0.0);
    let from_y_minus = REGIONS_HEIGHT / 2;
    let from_y = center.0.1 - from_y_minus.min(center.0.1);
    let from = RegionXy(Xy(from_x, from_y));

    let to_x_plus = REGIONS_WIDTH / 2;
    let to_x = (center.0.0 + to_x_plus).min(oc_root::REGIONS_WIDTH as u64 - 1);
    let to_y_plus = REGIONS_HEIGHT / 2;
    let to_y = (center.0.1 + to_y_plus).min(oc_root::REGIONS_HEIGHT as u64 - 1);
    let to = RegionXy(Xy(to_x, to_y));

    let (from_region_x, from_region_y) = (from.0.0, from.0.1);
    let (to_region_x, to_region_y) = (to.0.0, to.0.1);

    (from_region_x..=to_region_x)
        .flat_map(|x| (from_region_y..=to_region_y).map(move |y| RegionXy(Xy(x, y)).into()))
        .collect()
}
