use oc_utils::d2::Position;
use rkyv::Archive;

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
pub enum Order {
    Idle,
    MoveTo(Position),
}
