use bevy::prelude::*;
use oc_battle_gui::{ingame::FirstIngameEnter, network::output::ToServerEvent, states::Game};
use oc_network::ToServer;

#[cfg(feature = "test")]
use {
    oc_examples::tests::wall, oc_mod::Mod, oc_projectile::spawn::SpawnProjectile,
    std::path::PathBuf, std::time::Duration,
};

const TIMEOUT: Duration = Duration::from_secs(5);

#[cfg(feature = "test")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = wall::run(Some(Box::new(install))).unwrap();
    Ok(())
}

fn install(app: &mut bevy::app::App) {
    app..add_observer(on_first_ingame_enter);
}

fn end(mut commands: Commands, game: Res<Game>) {
    if game.started.elapsed() > TIMEOUT {
        commands.write_message(bevy::app::AppExit::from_code(1)); // TODO: have codes (x = timeout)
    }
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
