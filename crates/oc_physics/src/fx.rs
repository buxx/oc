use oc_mod::sound::SoundIndex;
use oc_root::geo::{WorldVec2, WorldVec3};
use oc_root::physics::Meters;
use rkyv::{Archive, Deserialize, Serialize};

#[derive(Archive, Deserialize, Serialize, Debug, PartialEq, Clone)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub enum Fx {
    Audio(Audio),
    Visual(Visual),
}

#[derive(Archive, Deserialize, Serialize, Debug, PartialEq, Clone)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub enum Audio {
    PlayWithGeo(SoundIndex, WorldVec3, SoundBus),
    Play(SoundIndex),
}

/// Mixing group of a sound: the gui applies one volume per bus
#[derive(Archive, Deserialize, Serialize, Debug, PartialEq, Clone, Copy, Eq, Hash)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub enum SoundBus {
    Fire,
    Effects,
    Explosion,
    Ui,
}

impl SoundBus {
    pub const ALL: [SoundBus; 4] = [
        SoundBus::Fire,
        SoundBus::Effects,
        SoundBus::Explosion,
        SoundBus::Ui,
    ];
}

#[derive(Archive, Deserialize, Serialize, Debug, PartialEq, Clone)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub enum Visual {
    Explosion(WorldVec2, Meters),
    /// Light flash at explosion center, sized by the largest explosion rayon
    Flash(WorldVec2, Meters),
    /// Smoke cloud (smoke grenade): position, seconds it keeps full opacity, seconds it takes to
    /// fade after that
    Smoke(WorldVec2, f32, f32),
    #[cfg(feature = "debug")]
    ExplosionDebug(ExplosionDebug),
}

#[cfg(feature = "debug")]
#[derive(Archive, Deserialize, Serialize, Debug, PartialEq, Clone)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct ExplosionDebug {
    pub position: WorldVec2,
    pub kill_rayon: Meters,
    pub stunned_rayon: Meters,
    pub shrapnel_rayon: Meters,
    pub destruction_rayon: Meters,
    pub shrapnels: Vec<(WorldVec2, WorldVec2)>,
}
