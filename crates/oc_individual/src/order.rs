use derive_more::Deref;
use enum_type_derive::EnumType;
use oc_root::geo::WorldVec2;
use oc_utils::d2::Direction;
use rkyv::Archive;

use crate::IndividualIndex;

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
    Engage(IndividualIndex),
    Suppress(WorldVec2),
}

impl Order {
    /// Return position if order own it (moves / suppress)
    pub fn position(&self) -> Option<WorldVec2> {
        match self {
            Order::Idle | Order::Defend(_) | Order::Hide(_) | Order::Engage(_) => None,
            Order::MoveTo(position)
            | Order::MoveFastTo(position)
            | Order::SneakTo(position)
            | Order::Suppress(position) => Some(*position),
        }
    }

    /// Update position if order own one (moves / fires)
    pub fn set_position(&mut self, position: WorldVec2) {
        match self {
            Order::Idle | Order::Defend(_) | Order::Hide(_) | Order::Engage(_) => {}
            Order::MoveTo(position_)
            | Order::MoveFastTo(position_)
            | Order::SneakTo(position_)
            | Order::Suppress(position_) => *position_ = position,
        }
    }
}
