use oc_mod::sound::SoundIndex;
use oc_root::geo::WorldVec3;
use rkyv::{Archive, Deserialize, Serialize};

#[derive(Archive, Deserialize, Serialize, Debug, PartialEq, Clone)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub enum Fx {
    Audio(Audio),
}

#[derive(Archive, Deserialize, Serialize, Debug, PartialEq, Clone)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub enum Audio {
    PlayWithGeo(SoundIndex, WorldVec3),
    Play(SoundIndex),
}
