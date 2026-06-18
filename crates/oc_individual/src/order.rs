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

impl Order {
    /// A manner to consider two order as same without strict compare.
    /// Useful for gui to know if its same order whereas details (like angle, path, ...)
    pub fn equal(&self, other: &Order) -> bool {
        match self {
            Order::Idle => matches!(other, Self::Idle),
            Order::MoveTo(position) => {
                matches!(other, Self::MoveTo(other_position) if other_position == position)
            }
        }
    }
}
