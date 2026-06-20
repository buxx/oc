use oc_individual::squad::{Squad, SquadIndex};
use rkyv::Archive;

#[derive(
    Debug,
    Clone,
    Archive,
    rkyv::Deserialize,
    rkyv::Serialize,
    serde::Serialize,
    serde::Deserialize,
    PartialEq,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct WorldResume {
    pub squads: Vec<(SquadIndex, Squad)>,
}
