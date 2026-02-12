use std::ops::Deref;

use derive_more::Constructor;
use oc_utils::d2::Xy;
use rkyv::{Archive, Deserialize, Serialize};

use crate::behavior::Behavior;

pub mod behavior;
pub mod network;

#[derive(Archive, Deserialize, Serialize, Clone, Copy, Debug, PartialEq, Constructor)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct IndividualIndex(pub u64);

#[derive(Archive, Deserialize, Serialize, Debug, PartialEq, Constructor)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct Individual {
    pub xy: Xy,
    pub behavior: Behavior,
}

#[derive(Debug, Clone, Copy, Archive, Deserialize, Serialize, PartialEq)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub enum Update {
    UpdateXy(Xy),
    UpdateBehavior(Behavior),
}

impl Deref for IndividualIndex {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<usize> for IndividualIndex {
    fn from(value: usize) -> Self {
        Self(value as u64)
    }
}

impl From<u64> for IndividualIndex {
    fn from(value: u64) -> Self {
        Self(value)
    }
}
