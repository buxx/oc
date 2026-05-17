use derive_more::Constructor;
use oc_geo::tile::TileXy;
use oc_physics::line;
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
    step: u64,
}

// FIXME: on ne devrait pas simplement utiliser Bresahim2d où les pixels sont les coordonées de tuiles ?
impl<'a, F> PathBuilder<'a, F>
where
    F: Fn(Xy, f32) -> Vec<Step>,
{
    pub fn build_(&self, start: [f32; 3], end: [f32; 3]) -> Path {
        let mut opacity = CumulatedOpacity(0.);
        let mut tile = TileXy::from_([start[0], start[1]], self.w);
        let mut sections = vec![];
        let mut last = start;

        // println!("Steps::new::{start:?}->[{end:?}]");
        for step in line::Steps::new(
            self.w.world_width_pixels,
            self.w.world_height_pixels,
            1.0,
            self.step,
            self.w.geo_pixels_per_tile,
            start.into(),
            end.into(),
        ) {
            let (pos, step_xy) = match step {
                line::Step::First(pos, xy)
                | line::Step::Last(pos, xy)
                | line::Step::Inside(pos, xy) => (pos, xy),
                line::Step::Outside => break,
            };

            // println!("Steps::step::{pos:?}");
            if step_xy != tile.0 {
                // println!("Steps::step::newtile");

                let mut new_opacity = opacity.0;
                for obj in (self.at)(step_xy, pos[2]) {
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

                // println!("Steps::step::new_opacity {new_opacity}");
                if new_opacity != opacity.0 {
                    // let x = opacity.0;
                    // println!(
                    //     "Steps::step::new_opacity::change::Section::(opacity {x}) {section_start:?}->{last:?}"
                    // );
                    sections.push(Section {
                        start: last,
                        stop: pos,
                        opacity,
                        nature: Nature::Visibility,
                    });
                    last = pos;
                    opacity.0 = new_opacity.min(1.0);
                    if opacity.0 >= 1.0 {
                        break;
                    }
                }

                tile.0 = step_xy;
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
        let path = PathBuilder::new(&w, at, 0).build_(start, end);

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
        let path = PathBuilder::new(&w, at, 0).build_(start, end);

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
        let path = PathBuilder::new(&w, at, 0).build_(start, end);

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
