#[cfg(feature = "test")]
use {
    oc_examples::tests::wall,
    oc_geo::tile::WorldTileIndex,
    oc_mod::Mod,
    oc_physics::Event,
    oc_projectile::{ProjectileId, spawn::SpawnProjectile},
    oc_root::end::End,
    oc_world_server::state::ObjectId,
    std::path::PathBuf,
    std::time::Duration,
};

#[cfg(feature = "test")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
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
    let projectiles = vec![SpawnProjectile::new(
        weapon.index(),
        ammunition.index(),
        shot.index(),
        1,
        [0., 0., 15.],
        [50., 50., 15.],
    )];

    // TODO: precise conditions wich permit exit before timeout (ohysics event for example)
    let end = End::new(Some(Duration::from_secs(5)));
    let tracker = wall::run(projectiles, end).unwrap();
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

#[cfg(not(feature = "test"))]
fn main() {}
