use derive_more::Constructor;
use oc_mod::{ammunition::AmmunitionIndex, armament::ShotMode, weapons::WeaponIndex};
use rkyv::{Archive, Deserialize, Serialize};

#[derive(Debug, Clone, Archive, Deserialize, Serialize, PartialEq, Constructor)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct SpawnProjectile {
    pub weapon: WeaponIndex,
    pub ammunition: AmmunitionIndex,
    pub shot_mode: ShotMode,
    pub repeat: u8,
    pub from: [f32; 3],
    pub to: [f32; 3],
}
