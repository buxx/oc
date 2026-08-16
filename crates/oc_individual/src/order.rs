use derive_more::Deref;
use enum_type_derive::EnumType;
use oc_root::geo::WorldVec2;
use oc_utils::d2::Direction;
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
    SneakTo(WorldVec2),
    Defend(Direction),
    Hide(Direction),
}

impl Order {
    /// Return position if order own it (moves / fires)
    pub fn position(&self) -> Option<WorldVec2> {
        match self {
            Order::Idle | Order::Defend(_) | Order::Hide(_) => None,
            Order::MoveTo(position) | Order::MoveFastTo(position) | Order::SneakTo(position) => {
                Some(*position)
            }
        }
    }

    /// Update position if order own one (moves / fires)
    pub fn set_position(&mut self, position: WorldVec2) {
        match self {
            Order::Idle | Order::Defend(_) | Order::Hide(_) => {}
            Order::MoveTo(position_) | Order::MoveFastTo(position_) | Order::SneakTo(position_) => {
                *position_ = position
            }
        }
    }
}

impl OrderType {
    /// Transform into Order. `point` and `reference` are used to determine target or direction according
    /// to order type.
    pub fn into_order(&self, point: WorldVec2, reference: WorldVec2) -> Order {
        match self {
            OrderType::Idle => Order::Idle,
            OrderType::MoveTo => Order::MoveTo(point),
            OrderType::MoveFastTo => Order::MoveFastTo(point),
            OrderType::SneakTo => Order::SneakTo(point),
            OrderType::Defend => {
                let direction = Direction::from_points2d(reference.into(), point.into());
                Order::Defend(direction)
            }
            OrderType::Hide => {
                let direction = Direction::from_points2d(reference.into(), point.into());
                Order::Hide(direction)
            }
        }
    }
}
