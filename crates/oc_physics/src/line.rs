use bresenham::Bresenham;
use oc_root::{GEO_PIXELS_PER_TILE, WORLD_HEIGHT, WORLD_WIDTH};
use oc_utils::d2::Xy;

use crate::Laws;

pub struct Steps<'a> {
    laws: &'a Laws,
    bresenham: Bresenham,
    step: usize,
    x: f32,
    y: f32,
    tile: Xy,
    target: Option<[isize; 2]>,
    first: bool,
}

impl<'a> Steps<'a> {
    pub fn new(laws: &'a Laws, (from_x, from_y): (f32, f32), (to_x, to_y): (f32, f32)) -> Self {
        let start = (
            (from_x * laws.bresenham_precision) as isize,
            (from_y * laws.bresenham_precision) as isize,
        );
        let end = (
            (to_x * laws.bresenham_precision) as isize,
            (to_y * laws.bresenham_precision) as isize,
        );
        let tile = Xy(
            from_x as u64 / GEO_PIXELS_PER_TILE,
            from_y as u64 / GEO_PIXELS_PER_TILE,
        );
        let bresenham = Bresenham::new(start, end);
        let distance = Xy::from(start).distance(Xy::from(end));
        let step = (laws.bresenham_step).min(distance as usize);
        let target = Some([end.0, end.1]);

        Self {
            laws,
            bresenham,
            x: from_x,
            y: from_y,
            step,
            tile,
            target,
            first: true,
        }
    }
}

impl<'a> Iterator for Steps<'a> {
    type Item = ([f32; 2], Xy);

    fn next(&mut self) -> Option<Self::Item> {
        if self.first {
            self.first = false;
            let tile = Xy(
                (self.x / self.laws.bresenham_precision) as u64 / self.laws.pixels_per_tile,
                (self.y / self.laws.bresenham_precision) as u64 / self.laws.pixels_per_tile,
            );
            return Some((([self.x, self.y]), tile));
        }

        if let Some((x, y)) = self.bresenham.nth(self.step) {
            if x < 0 || y < 0 || x + 1 >= WORLD_WIDTH as isize || y + 1 >= WORLD_HEIGHT as isize {
                return None;
            }

            let tile = Xy(
                (x as f32 / self.laws.bresenham_precision) as u64 / self.laws.pixels_per_tile,
                (y as f32 / self.laws.bresenham_precision) as u64 / self.laws.pixels_per_tile,
            );
            if tile != self.tile {
                self.tile = tile;
            };

            self.x = (x as f32) / self.laws.bresenham_precision;
            self.y = (y as f32) / self.laws.bresenham_precision;

            return Some(([self.x as f32, self.y as f32], self.tile.clone()));
        }

        if let Some([x, y]) = self.target.take() {
            let (x, y) = (
                x as f32 / self.laws.bresenham_precision,
                y as f32 / self.laws.bresenham_precision,
            );
            let tile = Xy(
                x as u64 / self.laws.pixels_per_tile,
                y as u64 / self.laws.pixels_per_tile,
            );
            return Some((([x as f32, y as f32]), tile));
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
        let laws = Laws::default()
            .bresenham_precision(100.)
            .bresenham_step(250);
        let mut steps = Steps::new(&laws, (0., 0.), (10.0, 10.0));

        // When-Then
        assert_eq!(steps.next(), Some(([0.0, 0.0], Xy(0, 0))));
        assert_eq!(steps.next(), Some(([2.5, 2.5], Xy(0, 0))));
        assert_eq!(steps.next(), Some(([5.01, 5.01], Xy(1, 1)))); // :(
        assert_eq!(steps.next(), Some(([7.52, 7.52], Xy(1, 1)))); // :(
        assert_eq!(steps.next(), Some(([10.0, 10.0], Xy(2, 2))));
        assert_eq!(steps.next(), None);
    }
}
