use derive_more::{Deref, DerefMut};
use glam::Vec2;
use oc_root::{geo::WorldVec2, physics::MetersSeconds};
use oc_utils::d2::Direction;
use rkyv::{Archive, Deserialize, Serialize};

#[derive(Debug, Clone, Archive, Deserialize, Serialize, PartialEq)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub enum Intent {
    Idle(Direction),
    MoveTo(WorldVec2, MovePath),
    MoveFastTo(WorldVec2, MovePath),
    Defend(Direction),
    Hide(Direction),
}

#[derive(Debug, Clone, Archive, Deserialize, Serialize, PartialEq)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub enum Behavior {
    Idle(Direction),
    Walk(Direction),
    Run(Direction),
    Defend(Direction),
    Hide(Direction),
}

impl Intent {
    pub fn path(&self) -> Option<(WorldVec2, &MovePath)> {
        match self {
            Intent::Idle(_) | Intent::Defend(_) | Intent::Hide(_) => None,
            Intent::MoveTo(target, path) | Intent::MoveFastTo(target, path) => {
                Some((*target, path))
            }
        }
    }
}

impl Behavior {
    pub fn nominal_speed(&self) -> MetersSeconds {
        match self {
            Behavior::Idle(_) | Behavior::Defend(_) | Behavior::Hide(_) => MetersSeconds(0.0),
            Behavior::Walk(_) => MetersSeconds(1.0),
            Behavior::Run(_) => MetersSeconds(2.0),
        }
    }

    pub fn direction(&self) -> Direction {
        match self {
            Behavior::Idle(direction)
            | Behavior::Walk(direction)
            | Behavior::Run(direction)
            | Behavior::Defend(direction)
            | Behavior::Hide(direction) => *direction,
        }
    }

    pub fn velocity(&self) -> f32 {
        let direction = self.direction();
        let direction = Vec2::new(direction.x, direction.y);
        direction.length()
    }
}

#[derive(Debug, Clone, Deref, DerefMut, Archive, Deserialize, Serialize, PartialEq)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct MovePath(pub Vec<WorldVec2>);

#[cfg(feature = "polyanya")]
impl From<polyanya::Path> for MovePath {
    fn from(value: polyanya::Path) -> Self {
        let path = value.path.iter().map(|p| [p.x, p.y].into()).collect();
        Self(path)
    }
}
