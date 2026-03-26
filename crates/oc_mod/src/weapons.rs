use std::{ops::Deref, path::PathBuf};

use anyhow::Context;
use oc_root::physics::{MetersSeconds, Seconds};
use rkyv::Archive;
use strum_macros::EnumIter;
use thiserror::Error;

use crate::{
    Mod, PickSound,
    ammunition::IndexedAmmunition,
    armament::{IndexedShotMode, ShotMode, ShotModeIndex, ShotModeRaw},
    sound::SoundIndex,
};

pub const WEAPONS_RON: &str = "weapons.ron";

#[derive(
    Debug,
    Clone,
    Copy,
    Archive,
    rkyv::Deserialize,
    rkyv::Serialize,
    PartialEq,
    serde::Deserialize,
    serde::Serialize,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct WeaponIndex(pub u32);

impl Deref for WeaponIndex {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(
    Debug,
    Clone,
    Archive,
    rkyv::Deserialize,
    rkyv::Serialize,
    PartialEq,
    serde::Deserialize,
    serde::Serialize,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct IndexedWeapon(pub WeaponIndex, pub Weapon);

impl Deref for IndexedWeapon {
    type Target = Weapon;

    fn deref(&self) -> &Self::Target {
        &self.1
    }
}

impl IndexedWeapon {
    pub fn index(&self) -> WeaponIndex {
        self.0
    }

    pub fn inner(&self) -> &Weapon {
        &self.1
    }
}

#[derive(
    Debug,
    Clone,
    Archive,
    rkyv::Deserialize,
    rkyv::Serialize,
    PartialEq,
    serde::Deserialize,
    serde::Serialize,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub enum Weapon {
    Rifle(Rifle),
    MachineGun(MachineGun),
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub enum WeaponRaw {
    Rifle(RifleRaw),
    MachineGun(MachineGunRaw),
}

impl Into<Weapon> for WeaponRaw {
    fn into(self) -> Weapon {
        match self {
            WeaponRaw::Rifle(rifle) => Weapon::Rifle(rifle.into()),
            WeaponRaw::MachineGun(machine_gun) => Weapon::MachineGun(machine_gun.into()),
        }
    }
}

impl WeaponRaw {
    pub fn name(&self) -> &str {
        match self {
            WeaponRaw::Rifle(rifle) => &rifle.name,
            WeaponRaw::MachineGun(machine_gun) => &machine_gun.name,
        }
    }

    pub fn amunitions(&self) -> &Vec<String> {
        match self {
            WeaponRaw::Rifle(rifle) => &rifle.amunitions,
            WeaponRaw::MachineGun(machine_gun) => &machine_gun.amunitions,
        }
    }

    pub fn shots(&self) -> &Vec<ShotModeRaw> {
        match self {
            WeaponRaw::Rifle(rifle) => &rifle.shots,
            WeaponRaw::MachineGun(machine_gun) => &machine_gun.shots,
        }
    }
}

impl Weapon {
    // TODO: derive (idem amunitions)
    pub fn name(&self) -> &str {
        match self {
            Weapon::Rifle(rifle) => &rifle.name,
            Weapon::MachineGun(machine_gun) => &machine_gun.name,
        }
    }

    // TODO: derive (idem amunitions)
    pub fn is_type(&self, type_: WeaponType) -> bool {
        match self {
            Weapon::Rifle(_) => matches!(type_, WeaponType::Rifle),
            Weapon::MachineGun(_) => matches!(type_, WeaponType::MachineGun),
        }
    }

    pub fn ammunitions(&self) -> &Vec<IndexedAmmunition> {
        match self {
            Weapon::Rifle(rifle) => &rifle.amunitions,
            Weapon::MachineGun(machine_gun) => &machine_gun.amunitions,
        }
    }

    pub fn set_ammunitions(&mut self, value: Vec<IndexedAmmunition>) {
        match self {
            Weapon::Rifle(rifle) => rifle.amunitions = value,
            Weapon::MachineGun(machine_gun) => machine_gun.amunitions = value,
        }
    }

    pub fn interval(&self) -> Seconds {
        match self {
            Weapon::Rifle(rifle) => rifle.interval,
            Weapon::MachineGun(machine_gun) => machine_gun.interval,
        }
    }

    pub fn shots(&self) -> &Vec<IndexedShotMode> {
        match self {
            Weapon::Rifle(rifle) => &rifle.shots,
            Weapon::MachineGun(machine_gun) => &machine_gun.shots,
        }
    }

    pub fn velocity(&self) -> MetersSeconds {
        match self {
            Weapon::Rifle(rifle) => rifle.velocity,
            Weapon::MachineGun(machine_gun) => machine_gun.velocity,
        }
    }

    fn set_shots(&mut self, value: Vec<IndexedShotMode>) {
        match self {
            Weapon::Rifle(rifle) => rifle.shots = value,
            Weapon::MachineGun(machine_gun) => machine_gun.shots = value,
        }
    }

    pub fn shot(&self, index: ShotModeIndex) -> &ShotMode {
        match self {
            Weapon::Rifle(rifle) => &rifle.shots[index.0 as usize],
            Weapon::MachineGun(machine_gun) => &machine_gun.shots[index.0 as usize],
        }
    }
}

// TODO: derive (idem amunitions)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, EnumIter, Default)]
pub enum WeaponType {
    #[default]
    Rifle,
    MachineGun,
}

impl WeaponType {
    pub fn name(&self) -> &str {
        match self {
            WeaponType::Rifle => "Rifle",
            WeaponType::MachineGun => "Machine gun",
        }
    }
}

#[derive(
    Debug,
    Clone,
    Archive,
    rkyv::Deserialize,
    rkyv::Serialize,
    PartialEq,
    serde::Deserialize,
    serde::Serialize,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct Rifle {
    name: String,
    amunitions: Vec<IndexedAmmunition>,
    shots: Vec<IndexedShotMode>,
    interval: Seconds,
    velocity: MetersSeconds,
}

// TODO: Derive or macro to raw version
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct RifleRaw {
    name: String,
    amunitions: Vec<String>,
    shots: Vec<ShotModeRaw>,
    interval: Seconds,
    velocity: MetersSeconds,
}

impl Into<Rifle> for RifleRaw {
    fn into(self) -> Rifle {
        Rifle {
            name: self.name,
            amunitions: vec![],
            shots: vec![],
            interval: self.interval,
            velocity: self.velocity,
        }
    }
}

#[derive(
    Debug,
    Clone,
    Archive,
    rkyv::Deserialize,
    rkyv::Serialize,
    PartialEq,
    serde::Deserialize,
    serde::Serialize,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct MachineGun {
    name: String,
    amunitions: Vec<IndexedAmmunition>,
    shots: Vec<IndexedShotMode>,
    interval: Seconds,
    velocity: MetersSeconds,
}

// TODO: Derive or macro to raw version
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct MachineGunRaw {
    name: String,
    amunitions: Vec<String>,
    shots: Vec<ShotModeRaw>,
    interval: Seconds,
    velocity: MetersSeconds,
}

impl Into<MachineGun> for MachineGunRaw {
    fn into(self) -> MachineGun {
        MachineGun {
            name: self.name,
            amunitions: vec![],
            shots: vec![],
            interval: self.interval,
            velocity: self.velocity,
        }
    }
}

// TODO: use something generic here (bullet/weapon/etc)
pub fn load(path: &PathBuf, mod_: &Mod) -> Result<Vec<IndexedWeapon>, Error> {
    let path = path.join(WEAPONS_RON);
    let weapons = std::fs::read_to_string(&path);
    let weapons = weapons.context(format!("Read {}", path.display()))?;
    let weapons: Vec<WeaponRaw> = ron::from_str(&weapons)?;
    let weapons: Vec<Weapon> = weapons
        .into_iter()
        .map(|weapon_| {
            let amunitions = mod_
                .amunitions_from_names(weapon_.amunitions().clone())
                .map_err(|e| Error::AmunitionRef(weapon_.name().to_string(), e.to_string()))?
                .into_iter()
                .map(|a| a.clone())
                .collect();
            let shots = weapon_
                .shots()
                .iter()
                .map(|mode| mode.clone().resolve_(mod_))
                .collect::<Result<Vec<ShotMode>, super::Error>>()
                .map_err(|e| Error::ShotMode(weapon_.name().to_string(), e.to_string()))?
                .into_iter()
                .enumerate()
                .map(|(i, shot)| IndexedShotMode(ShotModeIndex(i as u32), shot))
                .collect();

            let mut weapon: Weapon = weapon_.into();

            weapon.set_ammunitions(amunitions);
            weapon.set_shots(shots);

            Ok(weapon)
        })
        .collect::<Result<Vec<Weapon>, Error>>()?;

    if weapons.is_empty() {
        return Err(Error::Empty);
    }

    let weapons = weapons
        .into_iter()
        .enumerate()
        .map(|(i, p)| IndexedWeapon(WeaponIndex(i as u32), p))
        .collect();

    Ok(weapons)
}

impl PickSound<(WeaponIndex, ShotModeIndex)> for Mod {
    fn pick_sound(&self, (weapon, shot): (WeaponIndex, ShotModeIndex)) -> Option<SoundIndex> {
        let weapon = self.weapon(weapon);
        let shot = &weapon.shots()[shot.0 as usize];
        let sounds = shot.sounds();
        let i = fastrand::usize(..sounds.len());
        sounds.get(i).cloned()
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Any(#[from] anyhow::Error),
    #[error("Format: {0}")]
    InvalidFormat(String),
    #[error("Format: {0}")]
    Format(#[from] ron::de::SpannedError),
    #[error("No weapons defined (require at least one)")]
    Empty,
    #[error("Amunition error ({0}): {1}")]
    Amunition(String, String),
    #[error("Amunition error ({0}): {1}")]
    AmunitionRef(String, String),
    #[error("Invalid shot mode ({0}): {1}")]
    ShotMode(String, String),
}
