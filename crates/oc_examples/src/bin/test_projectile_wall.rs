use bevy::prelude::*;
use oc_battle_gui::{ingame::FirstIngameEnter, network::output::ToServerEvent, states::Game};
use oc_network::ToServer;

#[cfg(feature = "test")]
use {
    oc_examples::tests::wall,
    oc_geo::tile::WorldTileIndex,
    oc_mod::Mod,
    oc_physics::Event,
    oc_projectile::{ProjectileId, spawn::SpawnProjectile},
    oc_world_server::state::ObjectId,
    std::path::PathBuf,
    std::time::Duration,
};

const TIMEOUT: Duration = Duration::from_secs(5);

#[cfg(feature = "test")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // TODO: precise conditions wich permit exit before timeout (ohysics event for example)
    let tracker = wall::run(Some(Box::new(install))).unwrap();
    let physics = tracker.take().physics.clone();

    assert_eq!(
        physics,
        vec![Event::Collision(
            ObjectId::Projectile(ProjectileId(0)),
            ObjectId::Tile(WorldTileIndex(44))
        )]
    );

    Ok(())
}

fn install(app: &mut bevy::app::App) {
    app.add_systems(bevy::app::Update, end)
        .add_observer(on_first_ingame_enter);
}

fn end(mut commands: Commands, game: Res<Game>) {
    if game.started.elapsed() > TIMEOUT {
        commands.write_message(bevy::app::AppExit::from_code(1)); // TODO: have codes (x = timeout)
    }
}

fn on_first_ingame_enter(_: On<FirstIngameEnter>, mut commands: Commands) {
    let mod_ = Mod::load(&PathBuf::from("mods/std1"), None).unwrap();

    let weapon = mod_
        .weapons
        .iter()
        .find(|w| w.name() == "MosinNagantM1924")
        .unwrap();
    let ammunition = weapon
        .ammunitions()
        .iter()
        .find(|a| a.name() == "762x54R")
        .unwrap();
    let shot = weapon
        .shots()
        .iter()
        .find(|s| s.name() == "Single")
        .unwrap();

    for spawn in vec![SpawnProjectile::new(
        weapon.index(),
        ammunition.index(),
        shot.index(),
        1,
        [0., 0., 15.],
        [50., 50., 15.],
    )] {
        commands.trigger(ToServerEvent(ToServer::SpawnProjectile(spawn)));
    }
}
