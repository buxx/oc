use rkyv::{Archive, Deserialize, Serialize};

use crate::{IndividualIndex, Update};

#[derive(Debug, Clone, Archive, Deserialize, Serialize, PartialEq)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub enum IndividualMessage {
    Effect(IndividualIndex, Update),
}
