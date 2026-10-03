use derive_more::Constructor;
use oc_root::geo::WorldVec2;
use oc_utils::polygon::Polygon;

#[cfg_attr(feature = "bevy", derive(bevy::prelude::Component))]
#[derive(Clone, Debug, Constructor)]
pub struct Interior {
    pub id: u32,
    pub start: WorldVec2,
    pub points: Vec<WorldVec2>,
}

impl Polygon for Interior {
    fn points(&self) -> &[WorldVec2] {
        &self.points
    }
}

// TODO: move this test in oc_utils::polygon
#[cfg(test)]
mod tests {
    use super::*;

    // Warning: IA generated test
    #[test]
    fn test_contains() {
        let interior = Interior::new(
            1,
            WorldVec2::new(10., 10.),
            vec![
                WorldVec2::new(10., 10.),
                WorldVec2::new(20., 10.),
                WorldVec2::new(20., 20.),
                WorldVec2::new(10., 20.),
            ],
        );
        assert!(interior.contains(15., 15.));
        assert!(!interior.contains(25., 15.));
        assert!(!interior.contains(15., 5.));
    }
}
