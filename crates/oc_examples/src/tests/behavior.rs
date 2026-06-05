use std::path::PathBuf;

use anyhow::Context;
use bevy::prelude::*;
use bon::builder;
use oc_battle_gui::{
    ingame::{FirstIngameEnter, individual::Status},
    states::Game,
};
use oc_geo::{
    region::WorldRegionIndex,
    tile::{TileXy, WorldTileIndex},
};
use oc_individual::{IndividualIndex, network::Individual, order::Order, squad::Squad};
use oc_root::{WcfgFrom, WorldConfig, physics::Meters};
use oc_utils::d2::Xy;
use oc_world::{meta::Meta, tile::Tile};

use crate::{run, snapshot::SnapshotBuilder};

#[builder]
pub fn run(test: bool, setup: Vec<([f32; 2], Order)>) -> Result<(), anyhow::Error> {
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
        .install(Box::new(move |app: &mut bevy::app::App| install(app, test)))
        .snapshot(snapshot);

    let _ = example.build().run()?;

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
                oc_individual::behavior::Behavior::Idle,
                vec![],
                oc_individual::Status::Operational,
                oc_individual::Gesture::Idle,
                oc_individual::behavior::Intent::Idle,
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

fn install(app: &mut bevy::app::App, test: bool) {
    if test {
        app.add_systems(
            Update,
            |mut commands: Commands,
             game: Res<Game>,
             individuals: Query<
                &Status,
                With<oc_battle_gui::entity::individual::IndividualIndex>,
            >| {
                // let timeout = game.started.elapsed() > Duration::from_secs(10);
                // let dead = individuals
                //     .iter()
                //     .find(|status| matches!(status.0, oc_individual::Status::Dead))
                //     .is_some();

                // if timeout || dead {
                //     commands.write_message(bevy::app::AppExit::from_code(0));
                // }
            },
        );
    };
    app.add_observer(on_first_ingame_enter);
}

fn on_first_ingame_enter(_: On<FirstIngameEnter>, mut commands: Commands) {
    // let mod_ = Mod::load(&PathBuf::from("mods/tests1"), None).unwrap();

    // let weapon1 = mod_.weapons.iter().find(|w| w.name() == "Weapon1").unwrap();
    // let ammunition = weapon1
    //     .ammunitions()
    //     .iter()
    //     .find(|a| a.name() == "Ammo1")
    //     .unwrap();
    // let shot = weapon1
    //     .shots()
    //     .iter()
    //     .find(|s| s.name() == "Single")
    //     .unwrap();

    // for (start, end) in vec![([220.0, 150.0, 5.0], [100.0, 150.0, 5.0])] {
    //     commands.trigger(ToServerEvent(ToServer::SpawnProjectile(
    //         SpawnProjectile::new(
    //             weapon1.index(),
    //             ammunition.index(),
    //             shot.index(),
    //             1,
    //             start,
    //             end,
    //         ),
    //     )));
    // }
}
