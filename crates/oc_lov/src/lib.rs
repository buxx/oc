use derive_more::Constructor;
use line_drawing::Bresenham3d;
use oc_geo::tile::TileXy;
use oc_root::{
    WcfgFrom, WorldConfig,
    opacity::{CumulatedOpacity, Opacity},
};
use oc_utils::d2::Xy;

#[derive(Debug, Constructor)]
pub struct PathBuilder<'a, F>
where
    F: Fn(Xy, f32) -> Vec<Step>,
{
    w: &'a WorldConfig,
    at: F,
}

impl<'a, F> PathBuilder<'a, F>
where
    F: Fn(Xy, f32) -> Vec<Step>,
{
    pub fn build_(&self, start: [f32; 3], end: [f32; 3]) -> Path {
        tracing::trace!(name="lov-path-build", start=?start, end=?end);
        let mut opacity = CumulatedOpacity(0.);
        let mut tile = TileXy::from_([start[0], start[1]], self.w);
        let mut sections = vec![];
        let mut last = start;

        let start_ = (start[0] as isize, start[1] as isize, start[2] as isize);
        let end_ = (end[0] as isize, end[1] as isize, end[2] as isize);

        for (pixel_x, pixel_y, pixel_z) in Bresenham3d::new(start_, end_) {
            let pixel = [pixel_x as f32, pixel_y as f32, pixel_z as f32];
            let xy = Xy::from_((pixel_x, pixel_y), self.w);

            if xy != tile.0 {
                let mut new_opacity = opacity.0;
                for obj in (self.at)(xy, pixel[2]) {
                    // if obj.solid {
                    //     sections.push(Section {
                    //         start: section_start,
                    //         stop: last,
                    //         opacity,
                    //         nature: Nature::Obstacle,
                    //     });
                    //     section_start = pos;
                    //     blocked = true;
                    //     break 'steps;
                    // }

                    new_opacity += obj.opacity.0;
                }

                if new_opacity != opacity.0 {
                    sections.push(Section {
                        start: last,
                        stop: pixel,
                        opacity,
                        nature: Nature::Visibility,
                    });
                    last = pixel;
                    opacity.0 = new_opacity.min(1.0);
                    if opacity.0 >= 1.0 {
                        break;
                    }
                }

                tile.0 = xy;
            }
        }

        if start != end {
            sections.push(Section {
                start: last,
                stop: end,
                opacity,
                nature: Nature::Visibility,
            });
        }

        Path { sections }
    }
}

#[derive(Debug, PartialEq)]
pub struct Path {
    pub sections: Vec<Section>,
}

#[derive(Debug, PartialEq)]
pub struct Section {
    pub start: [f32; 3],
    pub stop: [f32; 3],
    pub opacity: CumulatedOpacity,
    pub nature: Nature,
}

#[derive(Debug, PartialEq)]
pub enum Nature {
    Visibility,
    // Obstacle,
    // Unknown,
}

#[derive(Debug)]
pub struct Step {
    /// Used to compute new opacity (when not solid)
    pub opacity: Opacity,
    // /// Used to know if path is interupted here
    // pub solid: bool,
}

#[cfg(test)]
mod test {
    use oc_root::physics::Meters;

    use super::*;

    #[test]
    fn test_short_path() {
        // Given
        let w = WorldConfig::new(3, 1, Meters(0.1)).geo_pixels_per_tile(5);
        let start = [0., 0., 0.];
        let end = [14., 0., 0.];
        let at = |_, _| {
            vec![Step {
                opacity: Opacity(0.1),
                // solid: false,
            }]
        };

        // When
        let path = PathBuilder::new(&w, at).build_(start, end);

        // Then
        assert_eq!(
            path,
            Path {
                sections: vec![
                    Section {
                        start: [0., 0., 0.],
                        stop: [5., 0., 0.],
                        opacity: CumulatedOpacity(0.),
                        nature: Nature::Visibility
                    },
                    Section {
                        start: [5., 0., 0.],
                        stop: [10., 0., 0.],
                        opacity: CumulatedOpacity(0.1),
                        nature: Nature::Visibility
                    },
                    Section {
                        start: [10., 0., 0.],
                        stop: [14., 0., 0.],
                        opacity: CumulatedOpacity(0.2),
                        nature: Nature::Visibility
                    }
                ]
            }
        )
    }

    #[test]
    fn test_path_with_space() {
        // Given
        let w = WorldConfig::new(6, 1, Meters(0.1)).geo_pixels_per_tile(5);
        let start = [0., 0., 0.];
        let end = [29., 0., 0.];
        let at = |xy, _| match xy {
            Xy(1, 0) | Xy(4, 0) => vec![Step {
                opacity: Opacity(0.1),
                // solid: false,
            }],
            _ => vec![Step {
                opacity: Opacity(0.0),
                // solid: false,
            }],
        };

        // When
        let path = PathBuilder::new(&w, at).build_(start, end);

        // Then
        assert_eq!(
            path,
            Path {
                sections: vec![
                    Section {
                        start: [0., 0., 0.],
                        stop: [5., 0., 0.],
                        opacity: CumulatedOpacity(0.),
                        nature: Nature::Visibility
                    },
                    Section {
                        start: [5., 0., 0.],
                        stop: [20., 0., 0.],
                        opacity: CumulatedOpacity(0.1),
                        nature: Nature::Visibility
                    },
                    Section {
                        start: [20., 0., 0.],
                        stop: [29., 0., 0.],
                        opacity: CumulatedOpacity(0.2),
                        nature: Nature::Visibility
                    },
                ]
            }
        )
    }

    #[test]
    fn test_opaque_path() {
        // Given
        let w = WorldConfig::new(3, 1, Meters(0.1)).geo_pixels_per_tile(5);
        let start = [0., 0., 0.];
        let end = [14., 0., 0.];
        let at = |_, _| {
            vec![Step {
                opacity: Opacity(0.6),
                // solid: false,
            }]
        };

        // When
        let path = PathBuilder::new(&w, at).build_(start, end);

        // Then
        assert_eq!(
            path,
            Path {
                sections: vec![
                    Section {
                        start: [0., 0., 0.],
                        stop: [5., 0., 0.],
                        opacity: CumulatedOpacity(0.),
                        nature: Nature::Visibility
                    },
                    Section {
                        start: [5., 0., 0.],
                        stop: [10., 0., 0.],
                        opacity: CumulatedOpacity(0.6),
                        nature: Nature::Visibility
                    },
                    Section {
                        start: [10., 0., 0.],
                        stop: [14., 0., 0.],
                        opacity: CumulatedOpacity(1.0),
                        nature: Nature::Visibility
                    }
                ]
            }
        )
    }

    // #[test]
    // fn test_blocked_path() {
    //     // Given
    //     let w = WorldConfig::new(3, 1, Meters(0.1)).geo_pixels_per_tile(5);
    //     let start = [0., 0., 0.];
    //     let end = [14., 0., 0.];
    //     let at = |xy, _| match xy {
    //         Xy(1, 0) => vec![Step {
    //             opacity: Opacity(0.1),
    //             solid: true,
    //         }],
    //         _ => vec![Step {
    //             opacity: Opacity(0.1),
    //             solid: false,
    //         }],
    //     };

    //     // When
    //     let path = PathBuilder::new(&w, at, 0).build_(start, end);

    //     // Then
    //     assert_eq!(
    //         path,
    //         Path {
    //             sections: vec![
    //                 Section {
    //                     start: [0., 0., 0.],
    //                     stop: [4., 0., 0.],
    //                     opacity: CumulatedOpacity(0.),
    //                     nature: Nature::Obstacle
    //                 },
    //                 Section {
    //                     start: [5., 0., 0.],
    //                     stop: [14., 0., 0.],
    //                     opacity: CumulatedOpacity(0.),
    //                     nature: Nature::Unknown
    //                 }
    //             ]
    //         }
    //     )
    // }
}
