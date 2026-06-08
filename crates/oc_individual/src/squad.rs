use derive_more::Constructor;
use rkyv::Archive;

use crate::{IndividualIndex, order::Order};

#[derive(
    Archive,
    rkyv::Deserialize,
    rkyv::Serialize,
    serde::Serialize,
    serde::Deserialize,
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Constructor,
    Hash,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct SquadIndex(pub u64);

#[derive(
    Debug,
    Clone,
    Archive,
    rkyv::Deserialize,
    rkyv::Serialize,
    serde::Serialize,
    serde::Deserialize,
    PartialEq,
    Constructor,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct Squad {
    pub members: Vec<IndividualIndex>,
    pub orders: Vec<Order>,
}

#[derive(Debug, Clone, Archive, rkyv::Deserialize, rkyv::Serialize, PartialEq)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub enum Update {
    Accomplished,
}
