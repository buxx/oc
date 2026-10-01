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
    /// Bounding box size (same as cropped interior image size)
    pub fn size(&self) -> WorldVec2 {
        let max_x = self
            .points
            .iter()
            .map(|p| p.x)
            .fold(f32::NEG_INFINITY, f32::max);
        let max_y = self
            .points
            .iter()
            .map(|p| p.y)
            .fold(f32::NEG_INFINITY, f32::max);
        WorldVec2::new(
            max_x.ceil() - self.start.x.floor(),
            max_y.ceil() - self.start.y.floor(),
        )
    }

    /// Even-odd ray casting point-in-polygon test.
    pub fn contains(&self, x: f32, y: f32) -> bool {
        let poly = &self.points;
        let Some(mut j) = poly.len().checked_sub(1) else {
            return false;
        };
        let mut result = false;

        for i in 0..poly.len() {
            let (xi, yi) = (poly[i].x, poly[i].y);
            let (xj, yj) = (poly[j].x, poly[j].y);
            if (yi > y) != (yj > y) && x < (xj - xi) * (y - yi) / (yj - yi) + xi {
                // FIXME BS NOW NOW: !? not return directly ?
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
