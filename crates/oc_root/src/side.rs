use rkyv::Archive;

#[derive(
    Debug,
    Clone,
    Copy,
    Archive,
    rkyv::Deserialize,
    rkyv::Serialize,
    serde::Serialize,
    serde::Deserialize,
    PartialEq,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub enum Side {
    A,
    B,
}
