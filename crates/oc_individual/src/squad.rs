use derive_more::Constructor;
use glam::Vec2;
use oc_root::{WorldConfig, geo::WorldVec2, side::Side, y::V};
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
    /// Number of alive members
    pub actives: u8,
    /// Formation of squad (to place members)
    pub formation: SquadFormation,
    /// Order given to this squad.
    pub orders: Vec<Order>,
    /// Computed position of the squad (leader position)
    pub position: WorldVec2,
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
    SetPosition(WorldVec2),
    SetActives(u8),
    Accomplished,
}

#[derive(
    Debug,
    Clone,
    Copy,
    Archive,
    rkyv::Deserialize,
    rkyv::Serialize,
    serde::Serialize,
    serde::Deserialize,
    PartialEq,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub enum SquadFormation {
    /// Formation make a line, squad leader at center
    Line,
}

impl SquadFormation {
    pub fn positions(
        &self,
        w: &WorldConfig,
        v: V,
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

                let dir_x = f32::cos(angle.0);
                let dir_y = match v {
                    V::Server => f32::sin(angle.0),
                    V::Gui => -f32::sin(angle.0),
                };
                let space =
                    w.formation_tiles_between_positions as f32 * w.geo_pixels_per_tile as f32;

                // Leader is already at `reference`. Remaining members fan out
                // alternately left/right, one rank further out each pair:
                // -1, +1, -2, +2, -3, +3, ...
                for i in 1..count {
                    let rank = ((i + 1) / 2) as f32; // 1,1,2,2,3,3,...
                    let side = if i % 2 == 1 { -1.0 } else { 1.0 };
                    let offset = side * rank * space;

                    let x = reference.x + dir_x * offset;
                    let y = reference.y + dir_y * offset;
                    positions.push(Vec2::new(x, y));
                }
            }
        }

        positions
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oc_root::{physics::Meters, y::V};
    use oc_utils::d2::Direction;
    use rstest::rstest;

    #[rstest]
    //
    #[case(1, Angle::zero(), vec![Vec2::new(100., 100.)])]
    #[case(2, Angle::zero(), vec![Vec2::new(100., 100.), Vec2::new(80., 100.)])]
    #[case(3, Angle::zero(), vec![Vec2::new(100., 100.), Vec2::new(80., 100.), Vec2::new(120., 100.)])]
    #[case(4, Angle::zero(), vec![Vec2::new(100., 100.), Vec2::new(80., 100.), Vec2::new(120., 100.), Vec2::new(60., 100.)])]
    #[case(5, Angle::zero(), vec![Vec2::new(100., 100.), Vec2::new(80., 100.), Vec2::new(120., 100.), Vec2::new(60., 100.), Vec2::new(140., 100.)])]
    //
    #[case(1, Direction::EST.angle(V::Server), vec![Vec2::new(100., 100.)])]
    #[case(2, Direction::EST.angle(V::Server), vec![Vec2::new(100., 100.), Vec2::new(100., 80.)])]
    #[case(3, Direction::EST.angle(V::Server), vec![Vec2::new(100., 100.), Vec2::new(100., 80.), Vec2::new(100., 120.)])]
    fn test_formation_line_positions(
        #[case] count: usize,
        #[case] angle: Angle,
        #[case] expected: Vec<Vec2>,
    ) {
        // Given
        let w = WorldConfig::new(100, 100, Meters(0.1)).geo_pixels_per_tile(10);
        let reference = Vec2::new(100., 100.);
        let formation = SquadFormation::Line;

        // When
        let positions = formation.positions(&w, V::Gui, reference, angle, count);

        // Then
        assert_eq!(positions, expected);
    }
}
