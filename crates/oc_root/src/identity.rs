use rkyv::Archive;
use uuid::Uuid;

use crate::side::Side;

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
pub struct Identity {
    pub uuid: Uuid,
    pub side: Side,
}
