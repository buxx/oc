use derive_more::Constructor;
use line_drawing::Bresenham3d;
use oc_geo::tile::TileXy;
use oc_root::{
    WcfgFrom, WorldConfig,
    geo::WorldVec3,
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
    pub fn build(&self, start: WorldVec3, end: WorldVec3, ignore: usize) -> Path {
        tracing::trace!(name="lov-path-build", start=?start, end=?end);
        let mut opacity = CumulatedOpacity(0.);
        let mut tile = TileXy::from_([start.x, start.y], self.w);
        let mut sections = vec![];
        let mut last = start;
        let mut ignored = 0usize;

        // Ceil to prevent > 0 been cast to 0
        let start_ = (
            start.x.ceil() as isize,
            start.y.ceil() as isize,
            start.z.ceil() as isize,
        );
        // Ceil to prevent > 0 been cast to 0
        let end_ = (
            end.x.ceil() as isize,
            end.y.ceil() as isize,
            end.z.ceil() as isize,
        );

        tracing::trace!(name="lov-path-build-bresenham3d", start_=?start_, end_=?end_);
        for (pixel_x, pixel_y, pixel_z) in Bresenham3d::new(start_, end_) {
            let pixel: WorldVec3 = [pixel_x as f32, pixel_y as f32, pixel_z as f32].into();
            let xy = Xy::from_((pixel_x, pixel_y), self.w);
            tracing::trace!(name="lov-path-build-bresenham3d-pixel", pixel=?pixel, xy=?xy);

            if xy != tile.0 {
                tile.0 = xy;

                if ignored < ignore {
                    tracing::trace!(name="lov-path-build-bresenham3d-ignore", pixel=?pixel, xy=?xy);
                    ignored += 1;
                    continue;
                }

                let mut new_opacity = opacity.0;
                for obj in (self.at)(xy, pixel.z) {
                    new_opacity += obj.opacity.0;
                    tracing::trace!(name="lov-path-build-bresenham3d-obj", pixel=?pixel, xy=?xy, obj=?obj, new_opacity=new_opacity);
                }

                if new_opacity != opacity.0 {
                    let section = Section {
                        start: last,
                        stop: pixel,
                        opacity,
                    };
                    tracing::trace!(name="lov-path-build-bresenham3d-section", pixel=?pixel, xy=?xy, section=?section);
                    sections.push(section);

                    last = pixel;
                    opacity.0 = new_opacity.min(1.0);
                    if opacity.0 >= 1.0 {
                        tracing::trace!(name="lov-path-build-bresenham3d-break", pixel=?pixel, xy=?xy, opacity=?opacity);
                        break;
                    }
                }
            }
        }

        if start != end {
            let section = Section {
                start: last,
                stop: end,
                opacity,
            };
            tracing::trace!(name="lov-path-build-bresenham3d-end-section", section=?section);
            sections.push(section);
        }

        Path { sections }
    }
}

#[derive(Debug, PartialEq)]
pub struct Path {
    pub sections: Vec<Section>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Section {
    pub start: WorldVec3,
    pub stop: WorldVec3,
    pub opacity: CumulatedOpacity,
}

impl From<(WorldVec3, WorldVec3, CumulatedOpacity)> for Section {
    fn from(value: (WorldVec3, WorldVec3, CumulatedOpacity)) -> Self {
        Section {
            start: value.0,
            stop: value.1,
            opacity: value.2,
        }
    }
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
        let path = PathBuilder::new(&w, at).build(start.into(), end.into(), 0);

        // Then
        assert_eq!(
            path,
            Path {
                sections: vec![
                    Section {
                        start: [0., 0., 0.].into(),
                        stop: [5., 0., 0.].into(),
                        opacity: CumulatedOpacity(0.),
                    },
                    Section {
                        start: [5., 0., 0.].into(),
                        stop: [10., 0., 0.].into(),
                        opacity: CumulatedOpacity(0.1),
                    },
                    Section {
                        start: [10., 0., 0.].into(),
                        stop: [14., 0., 0.].into(),
                        opacity: CumulatedOpacity(0.2),
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
        let path = PathBuilder::new(&w, at).build(start.into(), end.into(), 0);

        // Then
        assert_eq!(
            path,
            Path {
                sections: vec![
                    Section {
                        start: [0., 0., 0.].into(),
                        stop: [5., 0., 0.].into(),
                        opacity: CumulatedOpacity(0.),
                    },
                    Section {
                        start: [5., 0., 0.].into(),
                        stop: [20., 0., 0.].into(),
                        opacity: CumulatedOpacity(0.1),
                    },
                    Section {
                        start: [20., 0., 0.].into(),
                        stop: [29., 0., 0.].into(),
                        opacity: CumulatedOpacity(0.2),
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
        let path = PathBuilder::new(&w, at).build(start.into(), end.into(), 0);

        // Then
        assert_eq!(
            path,
            Path {
                sections: vec![
                    Section {
                        start: [0., 0., 0.].into(),
                        stop: [5., 0., 0.].into(),
                        opacity: CumulatedOpacity(0.),
                    },
                    Section {
                        start: [5., 0., 0.].into(),
                        stop: [10., 0., 0.].into(),
                        opacity: CumulatedOpacity(0.6),
                    },
                    Section {
                        start: [10., 0., 0.].into(),
                        stop: [14., 0., 0.].into(),
                        opacity: CumulatedOpacity(1.0),
                    }
                ]
            }
        )
    }

    #[test]
    fn test_opaque_path_but_ignore() {
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
        let path = PathBuilder::new(&w, at).build(start.into(), end.into(), 999);

        // Then
        assert_eq!(
            path,
            Path {
                sections: vec![Section {
                    start: WorldVec3 {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0
                    },
                    stop: WorldVec3 {
                        x: 14.0,
                        y: 0.0,
                        z: 0.0
                    },
                    opacity: CumulatedOpacity(0.0),
                }]
            }
        )
    }
}
