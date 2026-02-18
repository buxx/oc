use rkyv::{Archive, Deserialize, Serialize};

use crate::{Individual as Individual_, IndividualIndex, Update};

#[derive(Debug, Clone, Archive, Deserialize, Serialize, PartialEq)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub enum Individual {
    Insert(IndividualIndex, Individual_),
    Update(IndividualIndex, Update),
}
