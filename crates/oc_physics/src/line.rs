use line_drawing::Bresenham3d;
use oc_root::WorldConfig;
use oc_utils::d2::Xy;

pub struct Steps<'a> {
    w: &'a WorldConfig,
    bresenham: Bresenham3d<isize>,
    step: u64,
    x: f32,
    y: f32,
    z: f32,
    tile: Xy,
    target: Option<[isize; 3]>,
    first: bool,
}

impl<'a> Steps<'a> {
    pub fn new(
        w: &'a WorldConfig,
        (from_x, from_y, from_z): (f32, f32, f32),
        (to_x, to_y, to_z): (f32, f32, f32),
    ) -> Self {
        let start = (
            (from_x * w.geo_bresenham_precision) as isize,
            (from_y * w.geo_bresenham_precision) as isize,
            (from_z * w.geo_bresenham_precision) as isize,
        );
        let end = (
            (to_x * w.geo_bresenham_precision) as isize,
            (to_y * w.geo_bresenham_precision) as isize,
            (to_z * w.geo_bresenham_precision) as isize,
        );
        let tile = Xy(
            from_x as u64 / w.geo_pixels_per_tile,
            from_y as u64 / w.geo_pixels_per_tile,
        );
        let bresenham = Bresenham3d::new(start, end);
        let distance = Xy::from(start).distance(Xy::from(end));
        let step = (w.geo_bresenham_step).min(distance as u64);
        let target = Some([end.0, end.1, end.2]);

        Self {
            w,
            bresenham,
            x: start.0 as f32,
            y: start.1 as f32,
            z: start.2 as f32,
            step,
            tile,
            target,
            first: true,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Step {
    First([f32; 3], Xy),
    Inside([f32; 3], Xy),
    Last([f32; 3], Xy),
    Outside,
}

impl<'a> Iterator for Steps<'a> {
    type Item = Step;

    fn next(&mut self) -> Option<Self::Item> {
        let world_width = self.w.world_width_pixels as f32 * self.w.geo_bresenham_precision;
        let world_height = self.w.world_height_pixels as f32 * self.w.geo_bresenham_precision;

        if self.first {
            self.first = false;

            let tile = Xy(
                (self.x / self.w.geo_bresenham_precision) as u64 / self.w.geo_pixels_per_tile,
                (self.y / self.w.geo_bresenham_precision) as u64 / self.w.geo_pixels_per_tile,
            );
            return Some(Step::First(
                [
                    self.x / self.w.geo_bresenham_precision,
                    self.y / self.w.geo_bresenham_precision,
                    self.z / self.w.geo_bresenham_precision,
                ],
                tile,
            ));
        }

        if let Some((x, y, z)) = self.bresenham.nth(self.step as usize) {
            // TODO: maximum z ?
            if x < 0 || y < 0 || x + 1 >= world_width as isize || y + 1 >= world_height as isize {
                return Some(Step::Outside);
            }

            let tile = Xy(
                (x as f32 / self.w.geo_bresenham_precision) as u64 / self.w.geo_pixels_per_tile,
                (y as f32 / self.w.geo_bresenham_precision) as u64 / self.w.geo_pixels_per_tile,
            );
            if tile != self.tile {
                self.tile = tile;
            };

            self.x = (x as f32) / self.w.geo_bresenham_precision;
            self.y = (y as f32) / self.w.geo_bresenham_precision;
            self.z = (z as f32) / self.w.geo_bresenham_precision;

            return Some(Step::Inside([self.x, self.y, self.z], self.tile));
        }

        if let Some([x, y, z]) = self.target.take() {
            let (x, y, z) = (
                x as f32 / self.w.geo_bresenham_precision,
                y as f32 / self.w.geo_bresenham_precision,
                z as f32 / self.w.geo_bresenham_precision,
            );
            let tile = Xy(
                x as u64 / self.w.geo_pixels_per_tile,
                y as u64 / self.w.geo_pixels_per_tile,
            );
            return Some(Step::Last([x, y, z], tile));
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_steps_in_rectiline_line() {
        // Given
        let w = WorldConfig::new(1000, 1000)
            .geo_bresenham_precision(100.)
            .geo_bresenham_step(250);
        let mut steps = Steps::new(&w, (0., 0., 0.), (10.0, 10.0, 0.));

        // When-Then
        assert_eq!(steps.next(), Some(Step::First([0.0, 0.0, 0.0], Xy(0, 0))));
        assert_eq!(steps.next(), Some(Step::Inside([2.5, 2.5, 0.0], Xy(0, 0))));
        assert_eq!(
            steps.next(),
            Some(Step::Inside([5.01, 5.01, 0.0], Xy(1, 1)))
        );
        assert_eq!(
            steps.next(),
            Some(Step::Inside([7.52, 7.52, 0.0], Xy(1, 1)))
        );
        assert_eq!(steps.next(), Some(Step::Last([10.0, 10.0, 0.0], Xy(2, 2))));
        assert_eq!(steps.next(), None);
    }

    #[test]
    fn test_steps_in_diag() {
        // Given
        let w = WorldConfig::new(1000, 1000);
        let mut steps = Steps::new(&w, (10., 10., 0.), (15.0, 15.0, 0.));

        // When-Then
        assert_eq!(steps.next(), Some(Step::First([10.0, 10.0, 0.], Xy(2, 2))));
        assert_eq!(steps.next(), Some(Step::Inside([12.5, 12.5, 0.], Xy(2, 2))));
        assert_eq!(steps.next(), Some(Step::Last([15., 15., 0.], Xy(3, 3))));
    }
}
