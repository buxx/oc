use oc_root::{
    GEO_BRESENHAM_PRECISION, GEO_BRESENHAM_STEP, GEO_PIXELS_PER_TILE, PHYSICS_COEFF_PER_TICK,
};
use oc_utils::d2::Xy;
use rkyv::{Archive, Deserialize, Serialize};
use std::ops::Deref;

use crate::collision::Material;

pub mod collision;
pub mod line;
pub mod translation;

pub struct Laws {
    /// For units in x/s, this value is coeff per tick, to obtain 1.0 after the number of tick done in one second
    pub tick_coeff: f32,
    pub bresenham_precision: f32,
    pub bresenham_step: usize,
    pub pixels_per_tile: u64,
}

impl Laws {
    pub fn tick_coeff(mut self, value: f32) -> Self {
        self.tick_coeff = value;
        self
    }

    pub fn bresenham_precision(mut self, tick_coeff: f32) -> Self {
        self.bresenham_precision = tick_coeff;
        self
    }

    pub fn bresenham_step(mut self, value: usize) -> Self {
        self.bresenham_step = value;
        self
    }

    pub fn pixels_per_tile(mut self, value: u64) -> Self {
        self.pixels_per_tile = value;
        self
    }
}

impl Default for Laws {
    fn default() -> Self {
        Self {
            tick_coeff: PHYSICS_COEFF_PER_TICK,
            bresenham_precision: GEO_BRESENHAM_PRECISION,
            bresenham_step: GEO_BRESENHAM_STEP,
            pixels_per_tile: GEO_PIXELS_PER_TILE,
        }
    }
}

#[derive(Archive, Deserialize, Serialize, Debug, PartialEq, Clone)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct MetersSeconds(pub f32);

impl Deref for MetersSeconds {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub trait Physic: Material {
    fn position(&self) -> &[f32; 2];
    fn xy(&self) -> &Xy;
    fn forces(&self) -> &Vec<Force>;
}

#[derive(Archive, Deserialize, Serialize, Debug, PartialEq, Clone)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub enum Force {
    Translation([f32; 2], MetersSeconds),
}

#[inline]
pub fn step<'a, O: Physic, T: Material + 'a, F>(
    laws: &Laws,
    object: &O,
    tiles: F,
) -> ([f32; 2], Vec<Force>)
where
    F: Fn(Xy) -> Option<&'a T>,
{
    let mut position = object.position().clone();
    let mut forces = vec![];
    tracing::trace!(name="physics-step-start", p=?position, forces=?object.forces());

    'forces: for force in object.forces() {
        match force {
            Force::Translation(direction, speed) => {
                let s = speed.0 * laws.tick_coeff;
                let [x, y] = position;
                let (x_, y_) = (x + direction[0] * s, y + direction[1] * s);
                let mut curent_tile = Xy(
                    x as u64 / GEO_PIXELS_PER_TILE,
                    y as u64 / GEO_PIXELS_PER_TILE,
                );

                for ([step_x, step_y], step_tile) in line::Steps::new(laws, (x, y), (x_, y_)) {
                    // Test new tile only when line on new tile
                    if step_tile != curent_tile {
                        let Some(tile) = tiles(step_tile) else {
                            // No tile means outside map
                            tracing::trace!(name="physics-step-translation-no-tile", p=?position, xy=?step_tile);
                            continue 'forces;
                        };

                        // Move is finished if its a solid
                        if tile.material().is_solid() {
                            // Do not keep this force by stoping this iteration
                            tracing::trace!(name="physics-step-translation-solid", p=?position, xy=?step_tile);
                            continue 'forces;
                        }
                    }

                    curent_tile = step_tile;
                    position = [step_x, step_y];
                    tracing::trace!(name="physics-step-translation-updated", p=?position, xy=?step_tile);
                }
            }
        }

        forces.push(force.clone());
    }

    (position, forces)
}

#[cfg(test)]
mod tests {
    use crate::collision::Materials;

    use super::*;

    struct MyObject([f32; 2], Xy, Vec<Force>);

    impl Physic for MyObject {
        fn position(&self) -> &[f32; 2] {
            &self.0
        }
        fn xy(&self) -> &Xy {
            &self.1
        }
        fn forces(&self) -> &Vec<Force> {
            &self.2
        }
    }

    impl Material for MyObject {
        fn material(&self) -> Materials {
            Materials::Traversable
        }
    }

    struct MyTile(Materials);

    impl Material for MyTile {
        fn material(&self) -> Materials {
            self.0
        }
    }

    #[test]
    fn test_unidirectional_translation() {
        // Given
        let laws = Laws::default().tick_coeff(0.5).bresenham_precision(100.);
        let direction = [1.0, 0.0]; // South
        let speed = MetersSeconds(1.0);
        let force = Force::Translation(direction, speed);
        let object = MyObject([0.0, 0.0], Xy(0, 0), vec![force]);
        let tiles = |_| Some(&MyTile(Materials::Traversable));

        // When
        let (new_position, _) = step(&laws, &object, tiles);

        // Then
        let expected_new_position = [0.5, 0.0];
        assert_eq!(new_position, expected_new_position);
    }

    #[test]
    fn test_unidirectional_translation_collision() {
        // Given
        let laws = Laws::default()
            .tick_coeff(0.5)
            .bresenham_precision(100.)
            .bresenham_step(250);
        let direction = [1.0, 0.0]; // South
        let speed = MetersSeconds(100.0);
        let force = Force::Translation(direction, speed);
        let object = MyObject([0.0, 0.0], Xy(0, 0), vec![force]);
        let tiles = |xy| {
            if xy == Xy(0, 0) {
                Some(&MyTile(Materials::Traversable))
            } else {
                Some(&MyTile(Materials::Solid))
            }
        };

        // When
        let (new_position, new_forces) = step(&laws, &object, tiles);

        // Then
        let expected_new_position = [2.5, 0.0];
        let expected_new_forces: Vec<Force> = vec![];
        assert_eq!(new_position, expected_new_position);
        assert_eq!(new_forces, expected_new_forces);
    }

    #[test]
    fn test_unidirectional_translation_high_speed() {
        // Given
        let laws = Laws::default().tick_coeff(0.5).bresenham_precision(100.);
        let direction = [1.0, 0.0]; // South
        let speed = MetersSeconds(10.0);
        let force = Force::Translation(direction, speed);
        let object = MyObject([0.0, 0.0], Xy(0, 0), vec![force]);
        let tiles = |_| Some(&MyTile(Materials::Traversable));

        // When
        let (new_position, _) = step(&laws, &object, tiles);

        // Then
        let expected_new_position = [5.0, 0.0];
        assert_eq!(new_position, expected_new_position);
    }

    #[test]
    fn test_unidirectional_translation_high_speed_collision() {
        // Given
        let laws = Laws::default()
            .tick_coeff(0.5)
            .bresenham_precision(100.)
            .bresenham_step(250);
        let direction = [1.0, 0.0]; // South
        let speed = MetersSeconds(100.0);
        let force = Force::Translation(direction, speed);
        let object = MyObject([0.0, 0.0], Xy(0, 0), vec![force]);
        let tiles = |xy| {
            if xy == Xy(0, 0) {
                Some(&MyTile(Materials::Traversable))
            } else {
                Some(&MyTile(Materials::Solid))
            }
        };

        // When
        let (new_position, new_forces) = step(&laws, &object, tiles);

        // Then
        let expected_new_position = [2.5, 0.0];
        let expected_new_forces: Vec<Force> = vec![];
        assert_eq!(new_position, expected_new_position);
        assert_eq!(new_forces, expected_new_forces);
    }

    #[test]
    fn test_bidirectional_translation() {
        // Given
        let laws = Laws::default().tick_coeff(0.5).bresenham_precision(100.);
        let direction = [1.0, 1.0]; // South
        let speed = MetersSeconds(1.0);
        let force = Force::Translation(direction, speed);
        let object = MyObject([0.0, 0.0], Xy(0, 0), vec![force]);
        let tiles = |_| Some(&MyTile(Materials::Traversable));

        // When
        let (new_position, _) = step(&laws, &object, tiles);

        // Then
        let expected_new_position = [0.5, 0.5];
        assert_eq!(new_position, expected_new_position);
    }
}
