// use line_drawing::Bresenham3d;
// use oc_utils::d2::Xy;

// pub struct Steps {
//     world_width_pixels: u64,
//     world_height_pixels: u64,
//     geo_pixels_per_tile: u64,
//     bresenham: Bresenham3d<isize>,
//     x: f32,
//     y: f32,
//     z: f32,
//     tile: Xy,
//     target: Option<[isize; 3]>,
//     first: bool,
//     outside: bool,
// }

// impl Steps {
//     pub fn new(
//         world_width_pixels: u64,
//         world_height_pixels: u64,
//         geo_pixels_per_tile: u64,
//         (from_x, from_y, from_z): (f32, f32, f32),
//         (to_x, to_y, to_z): (f32, f32, f32),
//     ) -> Self {
//         let start = (from_x as isize, from_y as isize, from_z as isize);
//         let end = (to_x as isize, to_y as isize, to_z as isize);
//         let tile = Xy(
//             from_x as u64 / geo_pixels_per_tile,
//             from_y as u64 / geo_pixels_per_tile,
//         );
//         let bresenham = Bresenham3d::new(start, end);
//         let target = Some([end.0, end.1, end.2]);

//         Self {
//             world_width_pixels,
//             world_height_pixels,
//             geo_pixels_per_tile,
//             bresenham,
//             x: start.0 as f32,
//             y: start.1 as f32,
//             z: start.2 as f32,
//             tile,
//             target,
//             first: true,
//             outside: false,
//         }
//     }
// }

// impl Iterator for Steps {
//     type Item = [f32; 3];

//     fn next(&mut self) -> Option<Self::Item> {
//         if self.outside {
//             return None;
//         }

//         let world_width = self.world_width_pixels as f32;
//         let world_height = self.world_height_pixels as f32;

//         if let Some((x, y, z)) = self.bresenham.next() {
//             // TODO: maximum z ?
//             if x < 0 || y < 0 || x > world_width as isize - 1 || y > world_height as isize - 1 {
//                 self.outside = true;
//                 return Some(Step::Outside);
//             }

//             let tile = Xy(
//                 x as f32 as u64 / self.geo_pixels_per_tile,
//                 y as f32 as u64 / self.geo_pixels_per_tile,
//             );
//             if tile != self.tile {
//                 self.tile = tile;
//             };

//             self.x = x as f32;
//             self.y = y as f32;
//             self.z = z as f32;

//             return Some(Step::Inside([self.x, self.y, self.z], self.tile));
//         }

//         None
//     }
// }

// #[cfg(test)]
// mod tests {
//     use oc_root::{WorldConfig, physics::Meters};

//     use super::*;

//     #[test]
//     fn test_steps_in_rectiline_line() {
//         // Given
//         let w = WorldConfig::new(1000, 1000, Meters(0.1));
//         let mut steps = Steps::new(
//             w.world_width_pixels,
//             w.world_height_pixels,
//             w.geo_pixels_per_tile,
//             (0., 0., 0.),
//             (5.0, 5.0, 0.),
//         );

//         // When-Then
//         assert_eq!(steps.next(), Some(Step::Inside([0.0, 0.0, 0.0], Xy(0, 0))));
//         assert_eq!(steps.next(), Some(Step::Inside([1.0, 1.0, 0.0], Xy(0, 0))));
//         assert_eq!(steps.next(), Some(Step::Inside([2.0, 2.0, 0.0], Xy(0, 0))));
//         assert_eq!(steps.next(), Some(Step::Inside([3.0, 3.0, 0.0], Xy(0, 0))));
//         assert_eq!(steps.next(), Some(Step::Inside([4.0, 4.0, 0.0], Xy(0, 0))));
//         assert_eq!(steps.next(), Some(Step::Inside([5.0, 5.0, 0.0], Xy(1, 1))));
//         assert_eq!(steps.next(), None);

//         // assert_eq!(steps.next(), Some(Step::Inside([2.5, 2.5, 0.0], Xy(0, 0))));
//         // assert_eq!(
//         //     steps.next(),
//         //     Some(Step::Inside([5.01, 5.01, 0.0], Xy(1, 1)))
//         // );
//         // assert_eq!(
//         //     steps.next(),
//         //     Some(Step::Inside([7.52, 7.52, 0.0], Xy(1, 1)))
//         // );
//         // assert_eq!(steps.next(), Some(Step::Last([10.0, 10.0, 0.0], Xy(2, 2))));
//         // assert_eq!(steps.next(), None);
//     }

//     #[test]
//     fn test_steps_in_diag() {
//         // Given
//         let w = WorldConfig::new(1000, 1000, Meters(0.1));
//         let mut steps = Steps::new(
//             w.world_width_pixels,
//             w.world_height_pixels,
//             w.geo_pixels_per_tile,
//             (10., 10., 0.),
//             (15.0, 15.0, 0.),
//         );

//         // When-Then
//         assert_eq!(steps.next(), Some(Step::First([10.0, 10.0, 0.], Xy(2, 2))));
//         assert_eq!(steps.next(), Some(Step::Inside([12.5, 12.5, 0.], Xy(2, 2))));
//         assert_eq!(steps.next(), Some(Step::Last([15., 15., 0.], Xy(3, 3))));
//     }

//     #[test]
//     fn test_steps_outside_world_on_last() {
//         // Given
//         let w = WorldConfig::new(10, 10, Meters(0.1)).geo_pixels_per_tile(5);
//         let steps = Steps::new(
//             w.world_width_pixels,
//             w.world_height_pixels,
//             w.geo_pixels_per_tile,
//             (45., 45., 0.),
//             (55.0, 55.0, 0.),
//         );

//         // When
//         let steps: Vec<Step> = steps.collect();

//         // Then (non-reg, bug was Last (with outwrold coordinates) given after Outside)
//         assert_eq!(
//             steps,
//             vec![
//                 Step::First([45.0, 45.0, 0.0,], Xy(9, 9,),),
//                 Step::Inside([47.5, 47.5, 0.0,], Xy(9, 9,),),
//                 Step::Outside,
//             ]
//         )
//     }
// }
