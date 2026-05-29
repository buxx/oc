use std::path::PathBuf;

use anyhow::Context;
use bevy::prelude::*;
use oc_battle_gui::{ingame::FirstIngameEnter, network::output::ToServerEvent};
use oc_examples::{logging, run, snapshot::SnapshotBuilder};
use oc_geo::{
    region::{RegionXy, WorldRegionIndex},
    tile::{TileXy, WorldTileIndex},
};
use oc_individual::{Individual, behavior::Behavior};
use oc_mod::Mod;
use oc_network::ToServer;
use oc_physics::Force;
use oc_projectile::{Projectile, bullet::Bullet, spawn::SpawnProjectile};
use oc_root::{
    WcfgFrom, WorldConfig,
    physics::{Meters, MetersSeconds},
    y::Y,
};
use oc_utils::d2::Xy;
use oc_world::{meta::Meta, tile::Tile};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    logging::setup_logging()?;

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
    let snapshot = SnapshotBuilder::new(map_, individuals, vec![]).build(w, &mod__)?;

    let example = run::Example::builder()
        .world(map)
        .mod_(mod_)
        .install(Box::new(install))
        .snapshot(snapshot);

    example.build().run()?;

    Ok(())
}

fn individuals(w: &WorldConfig, tiles: &Vec<Tile>) -> Vec<Individual> {
    let positions = vec![[150.0, 150.0, 0.0]];

    // TODO: avoid repetition with main()
    let meta = Meta::from_file(&PathBuf::from("examples/meadow1/meta.toml")).unwrap();
    let map_ = PathBuf::from("examples/meadow1");
    let map = oc_world::reader::MapReader::new(&map_).unwrap();
    let w = WorldConfig::new(
        map.width().unwrap() as u64,
        map.height().unwrap() as u64,
        Meters(meta.geo_meters_per_z),
    );

    positions
        .into_iter()
        .map(|p| {
            let tile_xy = TileXy(Xy(
                p[0] as u64 / w.geo_pixels_per_tile,
                p[1] as u64 / w.geo_pixels_per_tile,
            ));
            let tile = WorldTileIndex::from_(tile_xy, &w);

            Individual::new(
                p.to_gui_y(&w),
                tile,
                WorldRegionIndex(0),
                Behavior::Idle,
                vec![],
            )
        })
        .collect()
}

fn install(app: &mut bevy::app::App) {
    app.add_observer(on_first_ingame_enter);
}

fn on_first_ingame_enter(_: On<FirstIngameEnter>, mut commands: Commands) {
    let mod_ = Mod::load(&PathBuf::from("mods/tests1"), None).unwrap();

    let weapon1 = mod_.weapons.iter().find(|w| w.name() == "Weapon1").unwrap();
    let ammunition = weapon1
        .ammunitions()
        .iter()
        .find(|a| a.name() == "Ammo1")
        .unwrap();
    let shot = weapon1
        .shots()
        .iter()
        .find(|s| s.name() == "Single")
        .unwrap();

    for (start, end) in vec![([220.0, 150.0, 5.0], [100.0, 150.0, 5.0])] {
        commands.trigger(ToServerEvent(ToServer::SpawnProjectile(
            SpawnProjectile::new(
                weapon1.index(),
                ammunition.index(),
                shot.index(),
                1,
                start,
                end,
            ),
        )));
    }
}
