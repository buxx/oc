use derive_more::Constructor;
use oc_mod::{ammunition::AmmunitionIndex, armament::ShotModeIndex, weapons::WeaponIndex};
use oc_root::geo::WorldVec3;
use rkyv::{Archive, Deserialize, Serialize};

#[derive(Debug, Clone, Archive, Deserialize, Serialize, PartialEq, Constructor)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct SpawnProjectile {
    pub weapon: WeaponIndex,
    pub ammunition: AmmunitionIndex,
    pub shot: ShotModeIndex,
    pub repeat: u8,
    pub from: WorldVec3,
    pub to: WorldVec3,
}
