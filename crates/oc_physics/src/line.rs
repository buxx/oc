use line_drawing::Bresenham3d;
use oc_utils::d2::Xy;

pub struct Steps {
    world_width_pixels: u64,
    world_height_pixels: u64,
    geo_bresenham_precision: f32,
    geo_pixels_per_tile: u64,
    bresenham: Bresenham3d<isize>,
    step: u64,
    x: f32,
    y: f32,
    z: f32,
    tile: Xy,
    target: Option<[isize; 3]>,
    first: bool,
    outside: bool,
}

impl Steps {
    pub fn new(
        world_width_pixels: u64,
        world_height_pixels: u64,
        geo_bresenham_precision: f32,
        geo_bresenham_step: u64,
        geo_pixels_per_tile: u64,
        (from_x, from_y, from_z): (f32, f32, f32),
        (to_x, to_y, to_z): (f32, f32, f32),
    ) -> Self {
        let start = (
            (from_x * geo_bresenham_precision) as isize,
            (from_y * geo_bresenham_precision) as isize,
            (from_z * geo_bresenham_precision) as isize,
        );
        let end = (
            (to_x * geo_bresenham_precision) as isize,
            (to_y * geo_bresenham_precision) as isize,
            (to_z * geo_bresenham_precision) as isize,
        );
        let tile = Xy(
            from_x as u64 / geo_pixels_per_tile,
            from_y as u64 / geo_pixels_per_tile,
        );
        let bresenham = Bresenham3d::new(start, end);
        let distance = Xy::from(start).distance(Xy::from(end));
        let step = (geo_bresenham_step).min(distance as u64);
        let target = Some([end.0, end.1, end.2]);

        Self {
            world_width_pixels,
            world_height_pixels,
            geo_bresenham_precision,
            geo_pixels_per_tile,
            bresenham,
            x: start.0 as f32,
            y: start.1 as f32,
            z: start.2 as f32,
            step,
            tile,
            target,
            first: true,
            outside: false,
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

// FIXME: could we remove geo_bresenham_precision by considering stop working with f32 ? (avoid over pixel precision)
// Franchement voir pour remplacer tout l'algo pour un truc simpleeeee
impl Iterator for Steps {
    type Item = Step;

    fn next(&mut self) -> Option<Self::Item> {
        if self.outside {
            return None;
        }

        let world_width = self.world_width_pixels as f32 * self.geo_bresenham_precision;
        let world_height = self.world_height_pixels as f32 * self.geo_bresenham_precision;

        if self.first {
            self.first = false;

            if self.x < 0. || self.y < 0. || self.x > world_width - 1. || self.y > world_height - 1.
            {
                self.outside = true;
                return Some(Step::Outside);
            }

            let tile = Xy(
                (self.x / self.geo_bresenham_precision) as u64 / self.geo_pixels_per_tile,
                (self.y / self.geo_bresenham_precision) as u64 / self.geo_pixels_per_tile,
            );
            return Some(Step::First(
                [
                    self.x / self.geo_bresenham_precision,
                    self.y / self.geo_bresenham_precision,
                    self.z / self.geo_bresenham_precision,
                ],
                tile,
            ));
        }

        // println!("step {0}", self.step);
        if let Some((x, y, z)) = self.bresenham.nth(self.step as usize) {
            // println!("step => {x},{y},{z}");
            // TODO: maximum z ?
            if x < 0 || y < 0 || x > world_width as isize - 1 || y > world_height as isize - 1 {
                // println!("Outside ({world_width},{world_height})");
                self.outside = true;
                return Some(Step::Outside);
            }

            let tile = Xy(
                (x as f32 / self.geo_bresenham_precision) as u64 / self.geo_pixels_per_tile,
                (y as f32 / self.geo_bresenham_precision) as u64 / self.geo_pixels_per_tile,
            );
            if tile != self.tile {
                self.tile = tile;
            };

            self.x = (x as f32) / self.geo_bresenham_precision;
            self.y = (y as f32) / self.geo_bresenham_precision;
            self.z = (z as f32) / self.geo_bresenham_precision;

            return Some(Step::Inside([self.x, self.y, self.z], self.tile));
        }

        if let Some([x, y, z]) = self.target.take() {
            if x < 0 || y < 0 || x > world_width as isize - 1 || y > world_height as isize - 1 {
                self.outside = true;
                return Some(Step::Outside);
            }

            let (x, y, z) = (
                x as f32 / self.geo_bresenham_precision,
                y as f32 / self.geo_bresenham_precision,
                z as f32 / self.geo_bresenham_precision,
            );
            let tile = Xy(
                x as u64 / self.geo_pixels_per_tile,
                y as u64 / self.geo_pixels_per_tile,
            );
            return Some(Step::Last([x, y, z], tile));
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use oc_root::{WorldConfig, physics::Meters};

    use super::*;

    #[test]
    fn test_steps_in_rectiline_line() {
        // Given
        let w = WorldConfig::new(1000, 1000, Meters(0.1))
            .geo_bresenham_precision(100.)
            .geo_bresenham_step(250);
        let mut steps = Steps::new(
            w.world_width_pixels,
            w.world_height_pixels,
            w.geo_bresenham_precision,
            w.geo_bresenham_step,
            w.geo_pixels_per_tile,
            (0., 0., 0.),
            (10.0, 10.0, 0.),
        );

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
        let w = WorldConfig::new(1000, 1000, Meters(0.1));
        let mut steps = Steps::new(
            w.world_width_pixels,
            w.world_height_pixels,
            w.geo_bresenham_precision,
            w.geo_bresenham_step,
            w.geo_pixels_per_tile,
            (10., 10., 0.),
            (15.0, 15.0, 0.),
        );

        // When-Then
        assert_eq!(steps.next(), Some(Step::First([10.0, 10.0, 0.], Xy(2, 2))));
        assert_eq!(steps.next(), Some(Step::Inside([12.5, 12.5, 0.], Xy(2, 2))));
        assert_eq!(steps.next(), Some(Step::Last([15., 15., 0.], Xy(3, 3))));
    }

    #[test]
    fn test_steps_outside_world_on_last() {
        // Given
        let w = WorldConfig::new(10, 10, Meters(0.1))
            .geo_bresenham_precision(100.)
            .geo_bresenham_step(250)
            .geo_pixels_per_tile(5);
        let steps = Steps::new(
            w.world_width_pixels,
            w.world_height_pixels,
            w.geo_bresenham_precision,
            w.geo_bresenham_step,
            w.geo_pixels_per_tile,
            (45., 45., 0.),
            (55.0, 55.0, 0.),
        );

        // When
        let steps: Vec<Step> = steps.collect();

        // Then (non-reg, bug was Last (with outwrold coordinates) given after Outside)
        assert_eq!(
            steps,
            vec![
                Step::First([45.0, 45.0, 0.0,], Xy(9, 9,),),
                Step::Inside([47.5, 47.5, 0.0,], Xy(9, 9,),),
                Step::Outside,
            ]
        )
    }
}
