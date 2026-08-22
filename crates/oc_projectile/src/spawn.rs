use derive_more::Constructor;
#[cfg(feature = "debug")]
use oc_individual::IndividualIndex;
use oc_mod::{ammunition::AmmunitionIndex, armament::ShotModeIndex, weapons::WeaponIndex};
use oc_root::{geo::WorldVec3, side::Side};
use rkyv::{Archive, Deserialize, Serialize};

#[derive(Debug, Clone, Archive, Deserialize, Serialize, PartialEq, Constructor)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct SpawnProjectiles {
    pub weapon: WeaponIndex,
    pub ammunition: AmmunitionIndex,
    pub shot: ShotModeIndex,
    pub repeat: u8,
    pub from: WorldVec3,
    pub directions: Vec<WorldVec3>,
    pub side: Side,
    #[cfg(feature = "debug")]
    pub shooter: Option<IndividualIndex>,
}

#[derive(Debug, Clone, Archive, Deserialize, Serialize, PartialEq, Constructor)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct SpawnProjectile {
    pub weapon: WeaponIndex,
    pub ammunition: AmmunitionIndex,
    pub shot: ShotModeIndex,
    pub from: WorldVec3,
    pub direction: WorldVec3,
    pub side: Side,
    #[cfg(feature = "debug")]
    pub shooter: Option<IndividualIndex>,
}

impl SpawnProjectiles {
    pub fn from_spawns(spawns: &SpawnProjectiles, direction: WorldVec3) -> SpawnProjectile {
        SpawnProjectile {
            weapon: spawns.weapon.clone(),
            ammunition: spawns.ammunition.clone(),
            shot: spawns.shot.clone(),
            from: spawns.from.clone(),
            direction,
            side: spawns.side.clone(),
            #[cfg(feature = "debug")]
            shooter: spawns.shooter.clone(),
        }
    }
}
