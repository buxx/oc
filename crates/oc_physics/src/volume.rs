use rkyv::{Archive, Deserialize, Serialize};

// WARNING: this module has been AI generated

#[derive(Debug, Clone, Archive, Deserialize, Serialize, PartialEq)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub enum Volume {
    Point,
    Cube { width: f32, height: f32, depth: f32 },
}

impl Volume {
    pub fn collide(
        &self,
        x1: f32,
        y1: f32,
        z1: f32,
        other: &Self,
        x2: f32,
        y2: f32,
        z2: f32,
    ) -> bool {
        match (self, other) {
            (Volume::Point, Volume::Point) => x1 == x2 && y1 == y2 && z1 == z2,

            (
                Volume::Point,
                Volume::Cube {
                    width,
                    height,
                    depth,
                },
            )
            | (
                Volume::Cube {
                    width,
                    height,
                    depth,
                },
                Volume::Point,
            ) => {
                println!(
                    "{x1} >= {x2}
                    && {x1} <= {x2} + {width}
                    && {y1} >= {y2}
                    && {y1} <= {y2} + {height}
                    && {z1} >= {z2}
                    && {z1} <= {z2} + {depth}"
                );
                x1 >= x2
                    && x1 <= x2 + width
                    && y1 >= y2
                    && y1 <= y2 + height
                    // FIXME BS NOW: problem is here;
                    // 2026-03-22T21:26:02.098663Z TRACE name="physics-step-translation-test-collide-with" p=[199.92, 57.2, -0.13] xy=Xy(40, 11) o=Tile(WorldTileIndex(11040))
                    // 202.43 >= 200
                    //     && 202.43 <= 200 + 5
                    //     && 57.18 >= 55
                    //     && 57.18 <= 55 + 5
                    //     && -1.09 >= 0
                    //     && -1.09 <= 0 + -340282350000000000000000000000000000000
                    && z1 >= z2
                    && z1 <= z2 + depth
            }

            (
                Volume::Cube {
                    width: w1,
                    height: h1,
                    depth: d1,
                },
                Volume::Cube {
                    width: w2,
                    height: h2,
                    depth: d2,
                },
            ) => {
                x1 < x2 + w2
                    && x1 + w1 > x2
                    && y1 < y2 + h2
                    && y1 + h1 > y2
                    && z1 < z2 + d2
                    && z1 + d1 > z2
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
        let a = Volume::Point;
        let b = Volume::Point;
        assert!(a.collide(1.0, 1.0, 1.0, &b, 1.0, 1.0, 1.0));
    }

    #[test]
    fn test_point_point_different() {
        let a = Volume::Point;
        let b = Volume::Point;
        assert!(!a.collide(1.0, 1.0, 1.0, &b, 1.0, 2.0, 1.0));
    }

    // Point vs Cube
    #[test]
    fn test_point_inside_cube() {
        let p = Volume::Point;
        let c = Volume::Cube {
            width: 5.0,
            height: 5.0,
            depth: 5.0,
        };
        assert!(p.collide(2.0, 2.0, 2.0, &c, 0.0, 0.0, 0.0));
    }

    #[test]
    fn test_point_outside_cube() {
        let p = Volume::Point;
        let c = Volume::Cube {
            width: 5.0,
            height: 5.0,
            depth: 5.0,
        };
        assert!(!p.collide(6.0, 6.0, 6.0, &c, 0.0, 0.0, 0.0));
    }

    #[test]
    fn test_point_outside_cube_z_axis() {
        let p = Volume::Point;
        let c = Volume::Cube {
            width: 5.0,
            height: 5.0,
            depth: 5.0,
        };
        // x and y are inside, but z is out
        assert!(!p.collide(2.0, 2.0, 6.0, &c, 0.0, 0.0, 0.0));
    }

    #[test]
    fn test_point_on_cube_face() {
        let p = Volume::Point;
        let c = Volume::Cube {
            width: 5.0,
            height: 5.0,
            depth: 5.0,
        };
        assert!(p.collide(0.0, 0.0, 0.0, &c, 0.0, 0.0, 0.0));
    }

    #[test]
    fn test_point_on_cube_corner() {
        let p = Volume::Point;
        let c = Volume::Cube {
            width: 5.0,
            height: 5.0,
            depth: 5.0,
        };
        assert!(p.collide(5.0, 5.0, 5.0, &c, 0.0, 0.0, 0.0));
    }

    // Symmetry: Cube vs Point should mirror Point vs Cube
    #[test]
    fn test_cube_point_symmetry() {
        let p = Volume::Point;
        let c = Volume::Cube {
            width: 5.0,
            height: 5.0,
            depth: 5.0,
        };
        assert_eq!(
            p.collide(2.0, 2.0, 2.0, &c, 0.0, 0.0, 0.0),
            c.collide(0.0, 0.0, 0.0, &p, 2.0, 2.0, 2.0)
        );
    }

    // Cube vs Cube
    #[test]
    fn test_cubes_overlapping() {
        let a = Volume::Cube {
            width: 5.0,
            height: 5.0,
            depth: 5.0,
        };
        let b = Volume::Cube {
            width: 5.0,
            height: 5.0,
            depth: 5.0,
        };
        assert!(a.collide(0.0, 0.0, 0.0, &b, 3.0, 3.0, 3.0));
    }

    #[test]
    fn test_cubes_not_overlapping() {
        let a = Volume::Cube {
            width: 5.0,
            height: 5.0,
            depth: 5.0,
        };
        let b = Volume::Cube {
            width: 5.0,
            height: 5.0,
            depth: 5.0,
        };
        assert!(!a.collide(0.0, 0.0, 0.0, &b, 6.0, 6.0, 6.0));
    }

    #[test]
    fn test_cubes_separated_on_z_axis() {
        // x and y overlap, but z does not
        let a = Volume::Cube {
            width: 5.0,
            height: 5.0,
            depth: 5.0,
        };
        let b = Volume::Cube {
            width: 5.0,
            height: 5.0,
            depth: 5.0,
        };
        assert!(!a.collide(0.0, 0.0, 0.0, &b, 0.0, 0.0, 6.0));
    }

    #[test]
    fn test_cubes_touching_face() {
        // Shares a face but does not overlap — no collision
        let a = Volume::Cube {
            width: 5.0,
            height: 5.0,
            depth: 5.0,
        };
        let b = Volume::Cube {
            width: 5.0,
            height: 5.0,
            depth: 5.0,
        };
        assert!(!a.collide(0.0, 0.0, 0.0, &b, 5.0, 0.0, 0.0));
    }

    #[test]
    fn test_cubes_one_inside_other() {
        let outer = Volume::Cube {
            width: 10.0,
            height: 10.0,
            depth: 10.0,
        };
        let inner = Volume::Cube {
            width: 3.0,
            height: 3.0,
            depth: 3.0,
        };
        assert!(outer.collide(0.0, 0.0, 0.0, &inner, 2.0, 2.0, 2.0));
    }

    #[test]
    fn test_cubes_same_position() {
        let a = Volume::Cube {
            width: 5.0,
            height: 5.0,
            depth: 5.0,
        };
        let b = Volume::Cube {
            width: 5.0,
            height: 5.0,
            depth: 5.0,
        };
        assert!(a.collide(0.0, 0.0, 0.0, &b, 0.0, 0.0, 0.0));
    }

    #[test]
    fn test_cubes_overlap_symmetry() {
        let a = Volume::Cube {
            width: 5.0,
            height: 5.0,
            depth: 5.0,
        };
        let b = Volume::Cube {
            width: 5.0,
            height: 5.0,
            depth: 5.0,
        };
        assert_eq!(
            a.collide(0.0, 0.0, 0.0, &b, 3.0, 3.0, 3.0),
            b.collide(3.0, 3.0, 3.0, &a, 0.0, 0.0, 0.0)
        );
    }
}
