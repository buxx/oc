use derive_more::Constructor;
use oc_root::geo::WorldVec2;

#[cfg_attr(feature = "bevy", derive(bevy::prelude::Component))]
#[derive(Clone, Debug, Constructor)]
pub struct Interior {
    pub id: u32,
    pub start: WorldVec2,
    pub points: Vec<WorldVec2>,
}

impl Interior {
    // Warning: IA generated function
    pub fn contains(&self, x: f32, y: f32) -> bool {
        let Some(mut j) = self.points.len().checked_sub(1) else {
            return false;
        };

        let mut result = false;
        for i in 0..self.points.len() {
            let (xi, yi) = (self.points[i].x, self.points[i].y);
            let (xj, yj) = (self.points[j].x, self.points[j].y);

            if (yi > y) != (yj > y) && x < (xj - xi) * (y - yi) / (yj - yi) + xi {
                result = !result;
            }

            j = i;
        }

        result
    }
}

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
