use derive_more::{Deref, DerefMut};
use oc_root::{geo::WorldVec2, physics::MetersSeconds};
use oc_utils::d2::Direction;
use rkyv::{Archive, Deserialize, Serialize};

use crate::IndividualIndex;

#[derive(Debug, Clone, Archive, Deserialize, Serialize, PartialEq)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub enum Intent {
    Idle(Direction),
    MoveTo(WorldVec2, MovePath),
    MoveFastTo(WorldVec2, MovePath),
    SneakTo(WorldVec2, MovePath),
    Defend(Direction),
    Hide(Direction),
    Engage(IndividualIndex),
    Suppress(WorldVec2),
}

#[derive(Debug, Clone, Archive, Deserialize, Serialize, PartialEq)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub enum Behavior {
    Idle(Direction),
    Walk(Direction),
    Run(Direction),
    Crawl(Direction),
    Defend(Direction),
    Hide(Direction),
    Engage(IndividualIndex),
    Suppress(WorldVec2),
}

impl Intent {
    pub fn path(&self) -> Option<(WorldVec2, &MovePath)> {
        match self {
            Intent::Idle(_)
            | Intent::Defend(_)
            | Intent::Hide(_)
            | Intent::Engage(_)
            | Intent::Suppress(_) => None,
            Intent::MoveTo(target, path)
            | Intent::MoveFastTo(target, path)
            | Intent::SneakTo(target, path) => Some((*target, path)),
        }
    }
}

impl Behavior {
    pub fn nominal_speed(&self) -> MetersSeconds {
        match self {
            Behavior::Idle(_)
            | Behavior::Defend(_)
            | Behavior::Hide(_)
            | Behavior::Engage(_)
            | Behavior::Suppress(_) => MetersSeconds(0.0),
            Behavior::Walk(_) => MetersSeconds(1.0),
            Behavior::Run(_) => MetersSeconds(2.0),
            Behavior::Crawl(_) => MetersSeconds(0.35),
        }
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
