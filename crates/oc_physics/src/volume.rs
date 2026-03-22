use oc_root::GEO_PIXELS_PER_TILE;
use rkyv::{Archive, Deserialize, Serialize};

// WARNING: this module has been AI generated

#[derive(Debug, Clone, Archive, Deserialize, Serialize, PartialEq)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub enum Volume {
    Point,
    Cube { width: f32, height: f32, depth: f32 },
    TileGround,
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
                x1 >= x2
                    && x1 <= x2 + width
                    && y1 >= y2
                    && y1 <= y2 + height
                    && z1 >= z2
                    && z1 <= z2 + depth
            }

            (Volume::Point, Volume::TileGround) | (Volume::TileGround, Volume::Point) => {
                let (px, py, tx, ty) = match (self, other) {
                    (Volume::Point, Volume::TileGround) => (x1, y1, x2, y2),
                    _ => (x2, y2, x1, y1),
                };
                px >= tx && px <= tx + GEO_PIXELS_PER_TILE as f32 && py <= ty
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

            (
                Volume::Cube {
                    width: w1,
                    depth: d1,
                    ..
                },
                Volume::TileGround,
            )
            | (
                Volume::TileGround,
                Volume::Cube {
                    width: w1,
                    depth: d1,
                    ..
                },
            ) => {
                let (cx, cy, cw, cd, tx, ty) = match (self, other) {
                    (
                        Volume::Cube {
                            width: w, depth: d, ..
                        },
                        Volume::TileGround,
                    ) => (x1, y1, w, d, x2, y2),
                    _ => (x2, y2, w1, d1, x1, y1),
                };
                cx < tx + GEO_PIXELS_PER_TILE as f32
                    && cx + cw > tx
                    && cy <= ty
                    && z1 < z2 + GEO_PIXELS_PER_TILE as f32
                    && z1 + cd > z2
            }

            (Volume::TileGround, Volume::TileGround) => {
                x1 < x2 + GEO_PIXELS_PER_TILE as f32
                    && x1 + GEO_PIXELS_PER_TILE as f32 > x2
                    && z1 < z2 + GEO_PIXELS_PER_TILE as f32
                    && z1 + GEO_PIXELS_PER_TILE as f32 > z2
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Point vs Point ---

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

    // --- Point vs Cube ---

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

    // --- Cube vs Cube ---

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

    // --- Point vs TileGround ---

    #[test]
    fn test_point_on_tile_ground_surface() {
        let p = Volume::Point;
        let g = Volume::TileGround;
        // Point exactly at tile's y surface, within x range
        assert!(p.collide(16.0, 0.0, 16.0, &g, 0.0, 0.0, 0.0));
    }

    #[test]
    fn test_point_below_tile_ground() {
        let p = Volume::Point;
        let g = Volume::TileGround;
        // y below the surface — still collides (infinite downward)
        assert!(p.collide(16.0, -100.0, 16.0, &g, 0.0, 0.0, 0.0));
    }

    #[test]
    fn test_point_above_tile_ground() {
        let p = Volume::Point;
        let g = Volume::TileGround;
        // y above the tile surface — no collision
        assert!(!p.collide(16.0, 1.0, 16.0, &g, 0.0, 0.0, 0.0));
    }

    #[test]
    fn test_point_outside_tile_ground_x() {
        let p = Volume::Point;
        let g = Volume::TileGround;
        // x is out of the tile's width range
        assert!(!p.collide(
            GEO_PIXELS_PER_TILE as f32 + 1.0,
            0.0,
            16.0,
            &g,
            0.0,
            0.0,
            0.0
        ));
    }

    #[test]
    fn test_tile_ground_point_symmetry() {
        let p = Volume::Point;
        let g = Volume::TileGround;
        assert_eq!(
            p.collide(16.0, 0.0, 16.0, &g, 0.0, 0.0, 0.0),
            g.collide(0.0, 0.0, 0.0, &p, 16.0, 0.0, 16.0)
        );
    }

    // --- Cube vs TileGround ---

    #[test]
    fn test_cube_resting_on_tile_ground() {
        let c = Volume::Cube {
            width: 10.0,
            height: 10.0,
            depth: 10.0,
        };
        let g = Volume::TileGround;
        // Cube bottom (y) is exactly at tile surface
        assert!(c.collide(0.0, 0.0, 0.0, &g, 0.0, 0.0, 0.0));
    }

    #[test]
    fn test_cube_above_tile_ground() {
        let c = Volume::Cube {
            width: 10.0,
            height: 10.0,
            depth: 10.0,
        };
        let g = Volume::TileGround;
        // Cube bottom is above the tile surface — no collision
        assert!(!c.collide(0.0, 5.0, 0.0, &g, 0.0, 0.0, 0.0));
    }

    #[test]
    fn test_cube_outside_tile_ground_x() {
        let c = Volume::Cube {
            width: 5.0,
            height: 5.0,
            depth: 5.0,
        };
        let g = Volume::TileGround;
        // Cube is entirely to the right of the tile
        assert!(!c.collide(GEO_PIXELS_PER_TILE + 1.0, 0.0, 0.0, &g, 0.0, 0.0, 0.0));
    }

    #[test]
    fn test_cube_tile_ground_symmetry() {
        let c = Volume::Cube {
            width: 10.0,
            height: 10.0,
            depth: 10.0,
        };
        let g = Volume::TileGround;
        assert_eq!(
            c.collide(0.0, 0.0, 0.0, &g, 0.0, 0.0, 0.0),
            g.collide(0.0, 0.0, 0.0, &c, 0.0, 0.0, 0.0)
        );
    }

    // --- TileGround vs TileGround ---

    #[test]
    fn test_tile_grounds_same_position() {
        let a = Volume::TileGround;
        let b = Volume::TileGround;
        assert!(a.collide(0.0, 0.0, 0.0, &b, 0.0, 0.0, 0.0));
    }

    #[test]
    fn test_tile_grounds_adjacent_no_overlap() {
        let a = Volume::TileGround;
        let b = Volume::TileGround;
        // Placed exactly side by side — touching but not overlapping
        assert!(!a.collide(0.0, 0.0, 0.0, &b, GEO_PIXELS_PER_TILE as f32, 0.0, 0.0));
    }

    #[test]
    fn test_tile_grounds_overlapping() {
        let a = Volume::TileGround;
        let b = Volume::TileGround;
        assert!(a.collide(0.0, 0.0, 0.0, &b, 16.0, 0.0, 0.0));
    }
}
