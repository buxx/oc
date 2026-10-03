use oc_root::geo::WorldVec2;
use oc_utils::d2::Shape;
use oc_utils::polygon::Polygon;
use serde::{Deserialize, Serialize};

use crate::control::MapControl;
use crate::map::Map;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct FlagName(pub String);

#[derive(Clone)]
pub struct Flag {
    pub name: FlagName,
    pub start: WorldVec2,
    pub width: f32,
    pub height: f32,
    corners: [WorldVec2; 4],
}

impl Flag {
    pub fn new(name: FlagName, start: WorldVec2, width: f32, height: f32) -> Self {
        let (x, y) = (start.x, start.y);
        let corners = [
            WorldVec2::new(x, y),
            WorldVec2::new(x + width, y),
            WorldVec2::new(x + width, y + height),
            WorldVec2::new(x, y + height),
        ];

        Self {
            name,
            start,
            width,
            height,
            corners,
        }
    }

    pub fn shape(&self) -> Shape {
        Shape {
            top_left: self.corners[0].into(),
            top_right: self.corners[1].into(),
            bottom_right: self.corners[2].into(),
            bottom_left: self.corners[3].into(),
        }
    }
}

impl Polygon for Flag {
    fn points(&self) -> &[WorldVec2] {
        &self.corners
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum FlagOwnership {
    Nobody,
    A,
    B,
    Both,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct FlagsOwnership {
    ownerships: Vec<(FlagName, FlagOwnership)>,
}

impl FlagsOwnership {
    pub fn new(ownerships: Vec<(FlagName, FlagOwnership)>) -> Self {
        Self { ownerships }
    }

    pub fn empty() -> Self {
        Self { ownerships: vec![] }
    }

    pub fn from_control(map: &Map, a_control: &MapControl, b_control: &MapControl) -> Self {
        let mut ownerships = vec![];

        for flag in map.flags() {
            let is_a_control =
                map.one_of_spawn_zone_contains_flag(a_control.spawn_zone_names(), flag);
            let is_b_control =
                map.one_of_spawn_zone_contains_flag(b_control.spawn_zone_names(), flag);

            let flag_ownership = match (is_a_control, is_b_control) {
                (true, true) => FlagOwnership::Both,
                (true, false) => FlagOwnership::A,
                (false, true) => FlagOwnership::B,
                (false, false) => FlagOwnership::Nobody,
            };
            ownerships.push((flag.name.clone(), flag_ownership));
        }

        Self { ownerships }
    }

    pub fn ownerships(&self) -> &Vec<(FlagName, FlagOwnership)> {
        &self.ownerships
    }
}

// #[cfg(test)]
// pub mod test {
//     use rstest::*;

//     use crate::{decor::Decor, spawn::SpawnZone};

//     use super::*;
//     use std::path::PathBuf;

//     fn map(spawn_zones: Vec<SpawnZone>, flags: Vec<Flag>) -> Map {
//         Map::new(
//             PathBuf::from("."),
//             PathBuf::from("."),
//             PathBuf::from("."),
//             vec![],
//             spawn_zones,
//             10,
//             10,
//             Decor::new(vec![], vec![], Vec2::new(0., 0.)),
//             flags,
//             vec![],
//         )
//     }

//     #[cfg(test)]
//     #[fixture]
//     fn spawn_zones1() -> Vec<SpawnZone> {
//         vec![SpawnZone::new(
//             SpawnZoneName::North,
//             0.,
//             0.,
//             10.,
//             10.,
//             10.,
//             10.,
//         )]
//     }

//     #[cfg(test)]
//     #[fixture]
//     fn flag1() -> Flag {
//         Flag::new(FlagName("FlagName".to_string()), 1., 1., 8., 8.)
//     }

//     #[rstest]
//     #[case(MapControl::new(vec![SpawnZoneName::North]), MapControl::new(vec![]), FlagOwnership::A)]
//     #[case(MapControl::new(vec![]), MapControl::new(vec![SpawnZoneName::North]), FlagOwnership::B)]
//     #[case(MapControl::new(vec![SpawnZoneName::North]), MapControl::new(vec![SpawnZoneName::North]), FlagOwnership::Both)]
//     #[case(MapControl::new(vec![SpawnZoneName::North]), MapControl::new(vec![]), FlagOwnership::A)]
//     #[case(MapControl::new(vec![]), MapControl::new(vec![]), FlagOwnership::Nobody)]
//     fn flag_owned_by_a(
//         spawn_zones1: Vec<SpawnZone>,
//         flag1: Flag,
//         #[case] a_control: MapControl,
//         #[case] b_control: MapControl,
//         #[case] ownership: FlagOwnership,
//     ) {
//         // Given
//         let flags = vec![flag1];
//         let map = map(spawn_zones1, flags);

//         // When
//         let flags_ownership = FlagsOwnership::from_control(&map, &a_control, &b_control);

//         // Then
//         assert_eq!(
//             flags_ownership,
//             FlagsOwnership {
//                 ownerships: vec![(FlagName("FlagName".to_string()), ownership)]
//             }
//         )
//     }
// }
