use derive_more::Deref;
use oc_utils::d2::{Direction, Position};
use rkyv::{Archive, Deserialize, Serialize};

#[derive(Debug, Clone, Archive, Deserialize, Serialize, PartialEq)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub enum Intent {
    Idle(Direction),
    MoveTo(Position, MovePath),
}

#[derive(Debug, Clone, Archive, Deserialize, Serialize, PartialEq)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub enum Behavior {
    Idle(Direction),
    Walk(Direction),
}

#[derive(Debug, Clone, Deref, Archive, Deserialize, Serialize, PartialEq)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct MovePath(pub Vec<[f32; 2]>);

#[cfg(feature = "polyanya")]
impl From<polyanya::Path> for MovePath {
    fn from(value: polyanya::Path) -> Self {
        let path = value.path.iter().map(|p| [p.x, p.y]).collect();
        Self(path)
    }
}
