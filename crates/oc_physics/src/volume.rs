use rkyv::{Archive, Deserialize, Serialize};

// WARNING: this module has been AI generated

#[derive(Debug, Clone, Archive, Deserialize, Serialize, PartialEq)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub enum Volume {
    Point {
        x: f32,
        y: f32,
        z: f32,
    },
    Cube {
        x: f32,
        y: f32,
        z: f32,
        width: f32,
        height: f32,
        depth: f32,
    },
}

impl Volume {
    pub fn point_zero() -> Self {
        Self::Point {
            x: 0.,
            y: 0.,
            z: 0.,
        }
    }

    pub fn with_ref(mut self, value: [f32; 3]) -> Self {
        match &mut self {
            Volume::Point { x, y, z } => {
                *x = value[0];
                *y = value[1];
                *z = value[2];
            }
            Volume::Cube {
                x,
                y,
                z,
                width: _,
                height: _,
                depth: _,
            } => {
                *x = value[0];
                *y = value[1];
                *z = value[2];
            }
        };

        self
    }
}

impl Volume {
    pub fn collide(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Volume::Point {
                    x: x1,
                    y: y1,
                    z: z1,
                },
                Volume::Point {
                    x: x2,
                    y: y2,
                    z: z2,
                },
            ) => x1 == x2 && y1 == y2 && z1 == z2,

            (
                Volume::Point {
                    x: x1,
                    y: y1,
                    z: z1,
                },
                Volume::Cube {
                    x: x2,
                    y: y2,
                    z: z2,
                    width,
                    height,
                    depth,
                },
            )
            | (
                Volume::Cube {
                    x: x2,
                    y: y2,
                    z: z2,
                    width,
                    height,
                    depth,
                },
                Volume::Point {
                    x: x1,
                    y: y1,
                    z: z1,
                },
            ) => {
                x1 >= x2
                    && *x1 <= x2 + width
                    && y1 >= y2
                    && *y1 <= y2 + height
                    && z1 >= z2
                    && *z1 <= z2 + depth
            }

            (
                Volume::Cube {
                    x: x1,
                    y: y1,
                    z: z1,
                    width: w1,
                    height: h1,
                    depth: d1,
                },
                Volume::Cube {
                    x: x2,
                    y: y2,
                    z: z2,
                    width: w2,
                    height: h2,
                    depth: d2,
                },
            ) => {
                *x1 < x2 + w2
                    && x1 + w1 > *x2
                    && *y1 < y2 + h2
                    && y1 + h1 > *y2
                    && *z1 < z2 + d2
                    && z1 + d1 > *z2
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Point vs Point
    #[test]
    fn test_point_point_same() {
        let a = Volume::Point {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        };
        let b = Volume::Point {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        };
        assert!(a.collide(&b));
    }

    #[test]
    fn test_point_point_different() {
        let a = Volume::Point {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        };
        let b = Volume::Point {
            x: 1.0,
            y: 2.0,
            z: 1.0,
        };
        assert!(!a.collide(&b));
    }

    // Point vs Cube
    #[test]
    fn test_point_inside_cube() {
        let p = Volume::Point {
            x: 2.0,
            y: 2.0,
            z: 2.0,
        };
        let c = Volume::Cube {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            width: 5.0,
            height: 5.0,
            depth: 5.0,
        };
        assert!(p.collide(&c));
    }

    #[test]
    fn test_point_outside_cube() {
        let p = Volume::Point {
            x: 6.0,
            y: 6.0,
            z: 6.0,
        };
        let c = Volume::Cube {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            width: 5.0,
            height: 5.0,
            depth: 5.0,
        };
        assert!(!p.collide(&c));
    }

    #[test]
    fn test_point_outside_cube_z_axis() {
        let p = Volume::Point {
            x: 2.0,
            y: 2.0,
            z: 6.0,
        };
        let c = Volume::Cube {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            width: 5.0,
            height: 5.0,
            depth: 5.0,
        };
        assert!(!p.collide(&c));
    }

    #[test]
    fn test_point_on_cube_face() {
        let p = Volume::Point {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        let c = Volume::Cube {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            width: 5.0,
            height: 5.0,
            depth: 5.0,
        };
        assert!(p.collide(&c));
    }

    #[test]
    fn test_point_on_cube_corner() {
        let p = Volume::Point {
            x: 5.0,
            y: 5.0,
            z: 5.0,
        };
        let c = Volume::Cube {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            width: 5.0,
            height: 5.0,
            depth: 5.0,
        };
        assert!(p.collide(&c));
    }

    // Symmetry: Cube vs Point should mirror Point vs Cube
    #[test]
    fn test_cube_point_symmetry() {
        let p = Volume::Point {
            x: 2.0,
            y: 2.0,
            z: 2.0,
        };
        let c = Volume::Cube {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            width: 5.0,
            height: 5.0,
            depth: 5.0,
        };
        assert_eq!(p.collide(&c), c.collide(&p));
    }

    // Cube vs Cube
    #[test]
    fn test_cubes_overlapping() {
        let a = Volume::Cube {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            width: 5.0,
            height: 5.0,
            depth: 5.0,
        };
        let b = Volume::Cube {
            x: 3.0,
            y: 3.0,
            z: 3.0,
            width: 5.0,
            height: 5.0,
            depth: 5.0,
        };
        assert!(a.collide(&b));
    }

    #[test]
    fn test_cubes_not_overlapping() {
        let a = Volume::Cube {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            width: 5.0,
            height: 5.0,
            depth: 5.0,
        };
        let b = Volume::Cube {
            x: 6.0,
            y: 6.0,
            z: 6.0,
            width: 5.0,
            height: 5.0,
            depth: 5.0,
        };
        assert!(!a.collide(&b));
    }

    #[test]
    fn test_cubes_separated_on_z_axis() {
        let a = Volume::Cube {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            width: 5.0,
            height: 5.0,
            depth: 5.0,
        };
        let b = Volume::Cube {
            x: 0.0,
            y: 0.0,
            z: 6.0,
            width: 5.0,
            height: 5.0,
            depth: 5.0,
        };
        assert!(!a.collide(&b));
    }

    #[test]
    fn test_cubes_touching_face() {
        let a = Volume::Cube {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            width: 5.0,
            height: 5.0,
            depth: 5.0,
        };
        let b = Volume::Cube {
            x: 5.0,
            y: 0.0,
            z: 0.0,
            width: 5.0,
            height: 5.0,
            depth: 5.0,
        };
        assert!(!a.collide(&b));
    }

    #[test]
    fn test_cubes_one_inside_other() {
        let outer = Volume::Cube {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            width: 10.0,
            height: 10.0,
            depth: 10.0,
        };
        let inner = Volume::Cube {
            x: 2.0,
            y: 2.0,
            z: 2.0,
            width: 3.0,
            height: 3.0,
            depth: 3.0,
        };
        assert!(outer.collide(&inner));
    }

    #[test]
    fn test_cubes_same_position() {
        let a = Volume::Cube {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            width: 5.0,
            height: 5.0,
            depth: 5.0,
        };
        let b = Volume::Cube {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            width: 5.0,
            height: 5.0,
            depth: 5.0,
        };
        assert!(a.collide(&b));
    }

    #[test]
    fn test_cubes_overlap_symmetry() {
        let a = Volume::Cube {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            width: 5.0,
            height: 5.0,
            depth: 5.0,
        };
        let b = Volume::Cube {
            x: 3.0,
            y: 3.0,
            z: 3.0,
            width: 5.0,
            height: 5.0,
            depth: 5.0,
        };
        assert_eq!(a.collide(&b), b.collide(&a));
    }
}
