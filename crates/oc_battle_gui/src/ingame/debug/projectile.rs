use derive_more::Constructor;
use oc_mod::{ammunition::AmmunitionIndex, armament::ShotModeIndex, weapons::WeaponIndex};
use oc_root::physics::Meters;

#[derive(Debug, Clone, Constructor)]
pub struct SpawnProjectileProfile {
    pub weapon: WeaponIndex,
    pub ammunition: AmmunitionIndex,
    pub shot: ShotModeIndex,
    pub repeat: u8,
    pub plus_z: Meters,
}
