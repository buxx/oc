use rkyv::{Archive, Deserialize, Serialize};

use crate::{
    Individual as Individual_, IndividualIndex, Update,
    squad::{self, SquadIndex},
};

#[derive(Debug, Clone, Archive, Deserialize, Serialize, PartialEq)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub enum Individual {
    Insert(IndividualIndex, Individual_),
    Update(IndividualIndex, Update),
    Physics(IndividualIndex, oc_physics::update::Update),
    Forgot(IndividualIndex),
}

#[derive(Debug, Clone, Archive, Deserialize, Serialize, PartialEq)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub enum Squad {
    Update(SquadIndex, squad::Update),
}
