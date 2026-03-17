use rkyv::{Archive, Deserialize, Serialize};

// TODO: intoroduce 3D, this is a temporary volume for 2D
#[derive(Debug, Clone, Archive, Deserialize, Serialize, PartialEq)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub enum Volume {
    Point,
    Square2d { width: f32, height: f32 },
}

impl Volume {
    pub fn collide(&self, x1: f32, y1: f32, other: &Self, x2: f32, y2: f32) -> bool {
        match (self, other) {
            (Volume::Point, Volume::Point) => x1 == x2 && y1 == y2,

            (Volume::Point, Volume::Square2d { width, height })
            | (Volume::Square2d { width, height }, Volume::Point) => {
                x1 >= x2 && x1 <= x2 + width && y1 >= y2 && y1 <= y2 + height
            }

            (
                Volume::Square2d {
                    width: w1,
                    height: h1,
                },
                Volume::Square2d {
                    width: w2,
                    height: h2,
                },
            ) => x1 < x2 + w2 && x1 + w1 > x2 && y1 < y2 + h2 && y1 + h1 > y2,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Point vs Point
    #[test]
    fn test_point_point_same() {
        let a = Volume::Point;
        let b = Volume::Point;
        assert!(a.collide(1.0, 1.0, &b, 1.0, 1.0));
    }

    #[test]
    fn test_point_point_different() {
        let a = Volume::Point;
        let b = Volume::Point;
        assert!(!a.collide(1.0, 1.0, &b, 1.0, 2.0));
    }

    // Point vs Square
    #[test]
    fn test_point_inside_square() {
        let p = Volume::Point;
        let s = Volume::Square2d {
            width: 5.0,
            height: 5.0,
        };
        assert!(p.collide(2.0, 2.0, &s, 0.0, 0.0));
    }

    #[test]
    fn test_point_outside_square() {
        let p = Volume::Point;
        let s = Volume::Square2d {
            width: 5.0,
            height: 5.0,
        };
        assert!(!p.collide(6.0, 6.0, &s, 0.0, 0.0));
    }

    #[test]
    fn test_point_on_square_edge() {
        let p = Volume::Point;
        let s = Volume::Square2d {
            width: 5.0,
            height: 5.0,
        };
        assert!(p.collide(0.0, 0.0, &s, 0.0, 0.0));
    }

    #[test]
    fn test_point_on_square_corner() {
        let p = Volume::Point;
        let s = Volume::Square2d {
            width: 5.0,
            height: 5.0,
        };
        assert!(p.collide(5.0, 5.0, &s, 0.0, 0.0));
    }

    // Symmetry: Square vs Point should mirror Point vs Square
    #[test]
    fn test_square_point_symmetry() {
        let p = Volume::Point;
        let s = Volume::Square2d {
            width: 5.0,
            height: 5.0,
        };
        assert_eq!(
            p.collide(2.0, 2.0, &s, 0.0, 0.0),
            s.collide(0.0, 0.0, &p, 2.0, 2.0)
        );
    }

    // Square vs Square
    #[test]
    fn test_squares_overlapping() {
        let a = Volume::Square2d {
            width: 5.0,
            height: 5.0,
        };
        let b = Volume::Square2d {
            width: 5.0,
            height: 5.0,
        };
        assert!(a.collide(0.0, 0.0, &b, 3.0, 3.0));
    }

    #[test]
    fn test_squares_not_overlapping() {
        let a = Volume::Square2d {
            width: 5.0,
            height: 5.0,
        };
        let b = Volume::Square2d {
            width: 5.0,
            height: 5.0,
        };
        assert!(!a.collide(0.0, 0.0, &b, 6.0, 6.0));
    }

    #[test]
    fn test_squares_touching_edge() {
        // Shares an edge but does not overlap — no collision
        let a = Volume::Square2d {
            width: 5.0,
            height: 5.0,
        };
        let b = Volume::Square2d {
            width: 5.0,
            height: 5.0,
        };
        assert!(!a.collide(0.0, 0.0, &b, 5.0, 0.0));
    }

    #[test]
    fn test_squares_one_inside_other() {
        let outer = Volume::Square2d {
            width: 10.0,
            height: 10.0,
        };
        let inner = Volume::Square2d {
            width: 3.0,
            height: 3.0,
        };
        assert!(outer.collide(0.0, 0.0, &inner, 2.0, 2.0));
    }

    #[test]
    fn test_squares_same_position() {
        let a = Volume::Square2d {
            width: 5.0,
            height: 5.0,
        };
        let b = Volume::Square2d {
            width: 5.0,
            height: 5.0,
        };
        assert!(a.collide(0.0, 0.0, &b, 0.0, 0.0));
    }

    #[test]
    fn test_squares_overlap_symmetry() {
        let a = Volume::Square2d {
            width: 5.0,
            height: 5.0,
        };
        let b = Volume::Square2d {
            width: 5.0,
            height: 5.0,
        };
        assert_eq!(
            a.collide(0.0, 0.0, &b, 3.0, 3.0),
            b.collide(3.0, 3.0, &a, 0.0, 0.0)
        );
    }
}
