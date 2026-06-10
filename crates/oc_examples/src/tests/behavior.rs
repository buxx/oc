use std::path::PathBuf;

use anyhow::Context;
use bevy::prelude::*;
use bon::builder;
use oc_geo::{
    region::WorldRegionIndex,
    tile::{TileXy, WorldTileIndex},
};
use oc_individual::{IndividualIndex, order::Order, squad::Squad};
use oc_root::{WcfgFrom, WorldConfig, physics::Meters};
use oc_utils::d2::{Direction, Xy};
use oc_world::{meta::Meta, tile::Tile};
#[cfg(feature = "test")]
use oc_world_server::tracker::Tracker;

use crate::{run, snapshot::SnapshotBuilder};

type Install = Box<dyn Fn(&mut bevy::app::App)>;
#[cfg(feature = "test")]
type Track = Box<dyn Fn(Tracker)>;
#[cfg(not(feature = "test"))]
type Track = ();

#[builder]
pub fn run(setup: Vec<([f32; 2], Order)>, tests: (Install, Track)) -> Result<(), anyhow::Error> {
    let mod_ = PathBuf::from("mods/tests1");
    let mod__ = oc_mod::Mod::load(&mod_, None)?;
    let map = PathBuf::from("examples/meadow1");
    let meta = Meta::from_file(&map.join("meta.toml"))?;
    let map_ = oc_world::reader::MapReader::new(&map);
    let map_ = map_.context(format!("Read map_ {}", map.display()))?;
    let w = WorldConfig::new(
        map_.width().unwrap() as u64,
        map_.height().unwrap() as u64,
        Meters(meta.geo_meters_per_z),
    );
    let tiles = map_.tiles(&w, &mod__).unwrap();

    let individuals = individuals(&w, &tiles, &setup);
    let squads = squads(&w, &individuals, &setup);
    let snapshot = SnapshotBuilder::new(map_, individuals, squads, vec![]).build(w, &mod__)?;

    let example = run::Example::builder()
        .world(map)
        .mod_(mod_)
        // .install(Box::new(move |app: &mut bevy::app::App| {
        //     install(app)
        // }))
        .snapshot(snapshot);

    let example = example.install(tests.0);

    #[allow(unused)]
    let tracker = example.build().run()?;

    #[cfg(feature = "test")]
    {
        tests.1(tracker);
    }

    Ok(())
}

fn individuals(
    w: &WorldConfig,
    tiles: &Vec<Tile>,
    setup: &Vec<([f32; 2], Order)>,
) -> Vec<oc_individual::Individual> {
    setup
        .iter()
        .map(|(position, _)| {
            let tile_xy = TileXy(Xy(
                position[0] as u64 / w.geo_pixels_per_tile,
                position[1] as u64 / w.geo_pixels_per_tile,
            ));
            let tile_i = WorldTileIndex::from_(tile_xy, &w);
            let tile = &tiles[tile_i.0 as usize];
            let z = tile.z_pixels(w);
            let position = [position[0], position[1], z];

            // FIXME BS NOW: introduce `create` function to initialize fields ...
            oc_individual::Individual::new(
                position,
                tile_i,
                WorldRegionIndex(0),
                vec![],
                oc_individual::behavior::Behavior::Idle(Direction::NORTH),
                vec![],
                oc_individual::Status::Operational,
                oc_individual::Gesture::Idle(Direction::NORTH),
                oc_individual::behavior::Intent::Idle(Direction::NORTH),
            )
        })
        .collect()
}

fn squads(
    _w: &WorldConfig,
    _individuals: &Vec<oc_individual::Individual>,
    setup: &Vec<([f32; 2], Order)>,
) -> Vec<Squad> {
    // For this test, all individual are alone in their squad
    // Test of squad behavior is in other example
    setup
        .iter()
        .enumerate()
        .map(|(i, (_, order))| {
            let individual = IndividualIndex(i as u64);
            Squad {
                members: vec![individual],
                orders: vec![order.clone()],
            }
        })
        .collect()
}
