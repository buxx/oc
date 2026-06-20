use derive_more::Constructor;
use oc_root::side::Side;
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
)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct Squad {
    pub side: Side,
    pub members: Vec<IndividualIndex>,
    pub orders: Vec<Order>,
    pub position: [f32; 2],
}

#[derive(Debug, Clone, Archive, rkyv::Deserialize, rkyv::Serialize, PartialEq)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub enum Update {
    SetOrders(Vec<Order>),
    SetPosition([f32; 2]),
    Accomplished,
}
