use derive_more::Deref;
use enum_type_derive::EnumType;
use glam::Vec2;
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
    /// Direction the order implies, seen from `from`. `individual_position` gives the position of
    /// an individual (for engage). None if order implies no direction (idle).
    pub fn direction(
        &self,
        from: Vec2,
        individual_position: impl Fn(IndividualIndex) -> Vec2,
    ) -> Option<Direction> {
        match self {
            Order::Idle => None,
            Order::Defend(direction) | Order::Hide(direction) => Some(*direction),
            Order::MoveTo(target)
            | Order::MoveFastTo(target)
            | Order::SneakTo(target)
            | Order::Suppress(target) => Some(Direction::from_points2d(from, (*target).into())),
            Order::Engage(target) => {
                Some(Direction::from_points2d(from, individual_position(*target)))
            }
        }
    }

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

    pub fn hud_label(&self) -> &str {
        match self {
            Order::Idle => "Idle",
            Order::MoveTo(_) => "Move",
            Order::MoveFastTo(_) => "Move fast",
            Order::SneakTo(_) => "Sneak",
            Order::Defend(_) => "Defend",
            Order::Hide(_) => "Hide",
            Order::Engage(_) => "Engage",
            Order::Suppress(_) => "Suppress",
        }
    }

    /// Modify self with position if self is an order which accept it
    pub fn with_position(mut self, position: WorldVec2) -> Order {
        match &mut self {
            Order::MoveTo(p) | Order::MoveFastTo(p) | Order::SneakTo(p) | Order::Suppress(p) => {
                *p = position
            }
            Order::Idle | Order::Defend(_) | Order::Hide(_) | Order::Engage(_) => {}
        }

        self
    }
}
