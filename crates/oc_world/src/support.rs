use derive_more::{Constructor, Deref};
use std::time::{Duration, Instant};

use oc_root::{geo::WorldVec2, side::Side};

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    rkyv::Archive,
    rkyv::Deserialize,
    rkyv::Serialize,
    serde::Serialize,
    serde::Deserialize,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub enum SupportType {
    NavalBombard,
}

impl SupportType {
    pub fn name(&self) -> &'static str {
        match self {
            SupportType::NavalBombard => "Naval bombard",
        }
    }

    /// Delay between the beginning of the fight (or the request) and the support execution
    pub fn prepare(&self) -> Duration {
        match self {
            SupportType::NavalBombard => Duration::from_secs(15),
        }
    }
}

#[derive(
    Debug,
    Clone,
    Default,
    PartialEq,
    rkyv::Archive,
    rkyv::Deserialize,
    rkyv::Serialize,
    serde::Serialize,
    serde::Deserialize,
    Deref,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct Support(pub Vec<(SupportType, bool)>);

#[derive(
    Debug,
    Clone,
    Default,
    PartialEq,
    rkyv::Archive,
    rkyv::Deserialize,
    rkyv::Serialize,
    serde::Serialize,
    serde::Deserialize,
    Constructor,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct Supports {
    pub side_a: Support,
    pub side_b: Support,
}

impl Support {
    pub fn set_availability(&mut self, type_: SupportType, value: bool) {
        for (t, available) in self.0.iter_mut() {
            if *t == type_ {
                *available = value;
            }
        }
    }
}

impl Supports {
    pub fn side_mut(&mut self, side: Side) -> &mut Support {
        match side {
            Side::A => &mut self.side_a,
            Side::B => &mut self.side_b,
        }
    }

    pub fn side(&self, side: Side) -> &Support {
        match side {
            Side::A => &self.side_a,
            Side::B => &self.side_b,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingSupport {
    pub type_: SupportType,
    pub side: Side,
    pub position: WorldVec2,
    pub start: Option<Instant>,
}

impl PendingSupport {
    pub fn fresh(side: Side, type_: SupportType, position: WorldVec2) -> Self {
        Self {
            type_,
            side,
            position,
            start: None,
        }
    }

    pub fn type_(&self) -> SupportType {
        self.type_
    }

    pub fn side(&self) -> Side {
        self.side
    }

    /// Preparation time starts only when the fight begins
    pub fn is_started(&self) -> bool {
        self.start.is_some()
    }

    pub fn start(&mut self) {
        self.start = Some(Instant::now());
    }

    pub fn is_ready(&self) -> bool {
        self.start
            .map(|s| s.elapsed() >= self.type_.prepare())
            .unwrap_or_default()
    }
}
