use derive_more::Constructor;
use glam::Vec2;
use oc_root::{WorldConfig, side::Side};
use oc_utils::d2::Angle;
use rkyv::Archive;

use crate::{IndividualIndex, order::Order};

#[derive(
    Archive,
    rkyv::Deserialize,
    rkyv::Serialize,
    serde::Serialize,
    serde::Deserialize,
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Constructor,
    Hash,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct SquadIndex(pub u64);

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
pub struct Squad {
    /// Side of the squad
    pub side: Side,
    /// Individual identifiers in this squad. Integrity must be checked before game run
    pub members: Vec<IndividualIndex>,
    /// FIXME BS NOW: must be computed regurarly
    /// Number of alive members
    pub actives: u8,
    /// Order given to this squad.
    pub orders: Vec<Order>,
    /// FIXME BS NOW: must be computed regurarly
    /// Computed position of the squad (leader position)
    pub position: [f32; 2],
}

impl Squad {
    pub fn leader(&self) -> IndividualIndex {
        *self
            .members
            .first()
            .expect("We delegate insurance there is member at the start of program")
    }
}

#[derive(Debug, Clone, Archive, rkyv::Deserialize, rkyv::Serialize, PartialEq)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub enum Update {
    SetOrders(Vec<Order>),
    SetPosition([f32; 2]),
    SetActives(u8),
    Accomplished,
}

const TILES_BETWEEN_POSITIONS: f32 = 4.;

#[derive(Debug, Clone, Copy)]
pub enum SquadFormation {
    /// Formation make a line, squad leader at center
    Line,
}

impl SquadFormation {
    /// Return squad members positions (include reference) according to formation and squad
    /// leader position (reference point)
    pub fn positions(
        &self,
        w: &WorldConfig,
        reference: Vec2,
        angle: Angle,
        count: usize,
    ) -> Vec<Vec2> {
        let mut positions = Vec::with_capacity(count);
        positions.push(reference);

        match self {
            SquadFormation::Line => {
                if count == 0 {
                    return vec![reference];
                }

                let perp_x = f32::cos(angle.0);
                let perp_y = f32::sin(angle.0);
                let center = (count as f32 - 1.0) / 2.0;
                let space = TILES_BETWEEN_POSITIONS * w.geo_pixels_per_tile as f32;

                // minus one because leader position aleeady in vec
                for i in 0..count.max(1) - 1 {
                    let offset = (i as f32 - center) * space;
                    let x = reference.x + perp_x * offset;
                    let y = reference.y + perp_y * offset;
                    let position = Vec2::new(x, y);

                    positions.push(position);
                }
            }
        }

        positions
    }
}
