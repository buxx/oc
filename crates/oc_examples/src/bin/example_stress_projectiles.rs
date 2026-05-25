use std::path::PathBuf;

use anyhow::Context;
use bevy::prelude::*;
use oc_battle_gui::{
    ingame::{FirstIngameEnter, camera::move_::CenterCameraOn},
    network::output::ToServerEvent,
};
use oc_examples::{logging, run, snapshot::SnapshotBuilder};
use oc_mod::Mod;
use oc_network::ToServer;
use oc_projectile::spawn::SpawnProjectile;
use oc_root::{WorldConfig, physics::Meters};
use oc_world::meta::Meta;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    logging::setup_logging()?;

    let mod_ = PathBuf::from("mods/tests1");
    let mod__ = Mod::load(&mod_, None)?;
    let meta = Meta::from_file(&PathBuf::from("examples/world1/meta.toml"))?;
    let map_ = PathBuf::from("examples/world1");
    let map = oc_world::reader::MapReader::new(&map_);
    let map = map.context(format!("Read map {}", map_.display()))?;
    let w = WorldConfig::new(
        map.width().unwrap() as u64,
        map.height().unwrap() as u64,
        Meters(meta.geo_meters_per_z),
    );
    let snapshot = SnapshotBuilder::new(map, vec![], vec![]).build(w, &mod__)?;

    let example = run::Example::builder()
        .world(PathBuf::from("examples/world1"))
        .mod_(mod_)
        .install(Box::new(install))
        .snapshot(snapshot);
    let _ = example.build().run()?;

    Ok(())
}

fn install(app: &mut bevy::app::App) {
    app.add_observer(on_first_ingame_enter);
}

fn on_first_ingame_enter(_: On<FirstIngameEnter>, mut commands: Commands) {
    // TODO: avoid repetition with main()
    let meta = Meta::from_file(&PathBuf::from("examples/world1/meta.toml")).unwrap();
    let map_ = PathBuf::from("examples/world1");
    let map = oc_world::reader::MapReader::new(&map_).unwrap();
    let w = WorldConfig::new(
        map.width().unwrap() as u64,
        map.height().unwrap() as u64,
        Meters(meta.geo_meters_per_z),
    );
    let mod_ = Mod::load(&PathBuf::from("mods/tests1"), None).unwrap();
    // Move at the center of the world
    let center_x = (map.width().unwrap() as f32 * w.geo_pixels_per_tile as f32) / 2.;
    let center_y = (map.height().unwrap() as f32 * w.geo_pixels_per_tile as f32) / 2.;
    commands.trigger(CenterCameraOn(Vec2::new(center_x, center_y)));

    let weapon1 = mod_.weapons.iter().find(|w| w.name() == "Weapon1").unwrap();
    let weapon2 = mod_.weapons.iter().find(|w| w.name() == "Weapon2").unwrap();
    let weapon3 = mod_.weapons.iter().find(|w| w.name() == "Weapon3").unwrap();
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
    let shot3 = weapon3
        .shots()
        .iter()
        .find(|s| s.name() == "Burst3")
        .unwrap();

    for spawn in vec![
        SpawnProjectile::new(
            weapon1.index(),
            ammunition.index(),
            shot.index(),
            10,
            [0., 0., 15.],
            [50., 50., 15.],
        ),
        SpawnProjectile::new(
            weapon2.index(),
            ammunition.index(),
            shot.index(),
            10,
            [10., 0., 15.],
            [60., 50., 15.],
        ),
        SpawnProjectile::new(
            weapon3.index(),
            ammunition.index(),
            shot3.index(),
            10,
            [20., 0., 15.],
            [70., 50., 15.],
        ),
    ] {
        commands.trigger(ToServerEvent(ToServer::SpawnProjectile(spawn)));
    }
}
