use oc_individual::squad::{Squad, SquadIndex};
use oc_root::battle::BattlePhase;

use crate::spawn::SpawnZoneName;

#[derive(
    Debug,
    Clone,
    rkyv::Archive,
    rkyv::Deserialize,
    rkyv::Serialize,
    serde::Serialize,
    serde::Deserialize,
)]
#[rkyv(derive(Debug))]
pub struct WorldResume {
    pub squads: Vec<(SquadIndex, Squad)>,
    pub side_a_spawns: Vec<SpawnZoneName>,
    pub side_b_spawns: Vec<SpawnZoneName>,
    pub phase: BattlePhase,
}
