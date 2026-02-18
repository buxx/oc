use rkyv::{Archive, Deserialize, Serialize};

use crate::{IndividualIndex, IndividualPublic, Update};

#[derive(Debug, Clone, Archive, Deserialize, Serialize, PartialEq)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub enum Individual {
    Insert(IndividualIndex, IndividualPublic),
    Update(IndividualIndex, Update),
}
