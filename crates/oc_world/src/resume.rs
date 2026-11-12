use oc_individual::Status;
use oc_individual::squad::{Squad, SquadIndex};
use oc_root::{battle::BattlePhase, side::Side};

use crate::{World, spawn::SpawnZoneName, support::SupportType};

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
    pub support: Vec<(SupportType, u8)>,
}

/// Battle end summary of one side
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Default,
    rkyv::Archive,
    rkyv::Deserialize,
    rkyv::Serialize,
    serde::Serialize,
    serde::Deserialize,
)]
#[rkyv(derive(Debug))]
pub struct EndResume {
    /// Individuals engaged in the battle
    pub engaged: u32,
    /// Engaged individuals dead in the battle
    pub dead: u32,
}

impl EndResume {
    pub fn from_world(world: &World, side: Side) -> Self {
        let individuals = || world.individuals.iter().filter(|i| i.side == side);
        let engaged = individuals().count() as u32;
        let dead = individuals()
            .filter(|i| matches!(i.status, Status::Dead))
            .count() as u32;

        Self { engaged, dead }
    }
}
