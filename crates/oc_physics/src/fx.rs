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
    PlayWithGeo(SoundIndex, WorldVec3),
    Play(SoundIndex),
}

#[derive(Archive, Deserialize, Serialize, Debug, PartialEq, Clone)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub enum Visual {
    Explosion(WorldVec2, Meters),
}
