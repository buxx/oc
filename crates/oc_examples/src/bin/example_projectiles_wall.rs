use bevy::prelude::*;
use oc_battle_gui::{ingame::FirstIngameEnter, network::output::ToServerEvent};
use oc_mod::Mod;
use oc_network::ToServer;
use std::path::PathBuf;

use {oc_examples::tests::wall, oc_projectile::spawn::SpawnProjectile};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = wall::run(Some(Box::new(install))).unwrap();
    Ok(())
}

fn install(app: &mut bevy::app::App) {
    app.add_observer(on_first_ingame_enter);
}

fn on_first_ingame_enter(_: On<FirstIngameEnter>, mut commands: Commands) {
    let mod_ = Mod::load(&PathBuf::from("mods/tests1"), None).unwrap();

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
