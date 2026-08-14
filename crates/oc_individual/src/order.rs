use derive_more::Deref;
use enum_type_derive::EnumType;
use oc_root::geo::WorldVec2;
use rkyv::Archive;

/// Index of squad order in squad order, by starting end
#[derive(Debug, Clone, Copy, Deref, Archive, rkyv::Deserialize, rkyv::Serialize, PartialEq)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct OrderIndex(pub u32);

#[derive(
    Debug,
    Clone,
    Archive,
    rkyv::Deserialize,
    rkyv::Serialize,
    serde::Serialize,
    serde::Deserialize,
    PartialEq,
    EnumType,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub enum Order {
    Idle,
    MoveTo(WorldVec2),
    MoveFastTo(WorldVec2),
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
            Order::MoveFastTo(position) => {
                matches!(other, Self::MoveFastTo(other_position) if other_position == position)
            }
        }
    }

    pub fn position(&self) -> Option<WorldVec2> {
        match self {
            Order::Idle => None,
            Order::MoveTo(position) | Order::MoveFastTo(position) => Some(*position),
        }
    }

    pub fn set_position(&mut self, position: WorldVec2) {
        match self {
            Order::Idle => todo!(),
            Order::MoveTo(position_) | Order::MoveFastTo(position_) => *position_ = position,
        }
    }
}

impl OrderType {
    pub fn into_order(&self, point: WorldVec2) -> Order {
        match self {
            OrderType::Idle => Order::Idle,
            OrderType::MoveTo => Order::MoveTo(point),
            OrderType::MoveFastTo => Order::MoveFastTo(point),
        }
    }
}
