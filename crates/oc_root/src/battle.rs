#[derive(
    Debug,
    Clone,
    Copy,
    rkyv::Archive,
    rkyv::Deserialize,
    serde::Deserialize,
    rkyv::Serialize,
    serde::Serialize,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub enum BattlePhase {
    Deployment,
    Fight,
}

impl BattlePhase {
    pub fn fight(&self) -> bool {
        match self {
            BattlePhase::Deployment => false,
            BattlePhase::Fight => true,
        }
    }
}
