use oc_root::{
    GEO_BRESENHAM_PRECISION, GEO_BRESENHAM_STEP, GEO_PIXELS_PER_METERS, GEO_PIXELS_PER_TILE,
    PHYSICS_COEFF_PER_TICK,
};
use oc_utils::d2::Xy;
use rkyv::{Archive, Deserialize, Serialize};
use std::ops::Deref;

use crate::{
    collision::{Collision, Material, Materials},
    volume::Volume,
};

pub mod collision;
pub mod line;
pub mod translation;
pub mod update;
pub mod volume;

pub struct Laws {
    /// For units in x/s, this value is coeff per tick, to obtain 1.0 after the number of tick done in one second
    pub tick_coeff: f32,
    pub bresenham_precision: f32,
    pub bresenham_step: usize,
    pub pixels_per_tile: u64,
    pub pixels_per_meters: f32,
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

    pub fn pixels_per_meters(mut self, value: f32) -> Self {
        self.pixels_per_meters = value;
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
            pixels_per_meters: GEO_PIXELS_PER_METERS,
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
    fn forces(&self) -> &Vec<Force>;
    fn volume(&self) -> &Volume;
}

pub trait UpdatePhysic: Physic + Material {
    fn set_position(&mut self, value: [f32; 2]);
    fn push_force(&mut self, value: Force);
    fn remove_force(&mut self, value: &Force);
    fn set_volume(&self, value: Volume);
}

// FIXME BS NOW; delete it ?
#[derive(Debug)]
pub struct Corps<I: Clone + std::fmt::Debug> {
    pub i: I,
    position: [f32; 2],
    forces: Vec<Force>,
    material: collision::Materials,
    volume: volume::Volume,
}

impl<I: Clone + std::fmt::Debug> Corps<I> {
    pub fn new(
        i: I,
        position: [f32; 2],
        forces: Vec<Force>,
        material: collision::Materials,
        volume: volume::Volume,
    ) -> Self {
        Self {
            i,
            position,
            forces,
            material,
            volume,
        }
    }
}

impl<I: Clone + std::fmt::Debug> Physic for Corps<I> {
    fn position(&self) -> &[f32; 2] {
        &self.position
    }

    fn forces(&self) -> &Vec<Force> {
        &self.forces
    }

    fn volume(&self) -> &Volume {
        &self.volume
    }
}

impl<I: Clone + std::fmt::Debug> collision::Material for Corps<I> {
    fn material(&self) -> collision::Materials {
        self.material
    }
}

// TODO: gravité
#[derive(Archive, Deserialize, Serialize, Debug, PartialEq, Clone)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub enum Force {
    Translation([f32; 2], MetersSeconds),
}

#[derive(Debug, Clone)]
pub enum Event<CL, CR> {
    NoTile,
    Collision(Collision<CL, CR>),
}

pub trait World<Z> {
    fn at(&self, xy: Xy) -> Vec<(Z, Box<dyn Physic>)>;
}

#[inline]
pub fn step<'a, I, O, F, Z>(
    laws: &Laws,
    object: (I, &'a O),
    at: F,
) -> ([f32; 2], Vec<Force>, Vec<Event<I, Z>>)
where
    I: Clone + std::fmt::Debug,
    O: Physic,
    // TODO: I failed to use references here, but I could be best for perfs ...
    F: Fn(Xy) -> Vec<(Z, Box<&'a dyn Physic>)>,
{
    let (i, object) = object;
    let volume = object.volume();
    let mut events = vec![];
    let mut position = object.position().clone();
    let mut forces = vec![];
    tracing::trace!(name="physics-step-start", p=?position, forces=?object.forces());

    'forces: for force in object.forces() {
        match force {
            Force::Translation(direction, speed) => {
                let speed = speed.0 * laws.tick_coeff;
                let pixels = speed * laws.pixels_per_meters as f32;
                let [x, y] = position;
                let (x_, y_) = (x + direction[0] * pixels, y + direction[1] * pixels);

                tracing::trace!(
                    name = "physics-step-translation-start",
                    x = x,
                    y = y,
                    x_ = x_,
                    y_ = y_,
                    speed = speed
                );

                let mut curent_tile = Xy(
                    x as u64 / GEO_PIXELS_PER_TILE,
                    y as u64 / GEO_PIXELS_PER_TILE,
                );

                for step in line::Steps::new(laws, (x, y), (x_, y_)) {
                    match step {
                        line::Step::First([step_x, step_y], step_tile)
                        | line::Step::Inside([step_x, step_y], step_tile)
                        | line::Step::Last([step_x, step_y], step_tile) => {
                            // Test new tile only when line on new tile
                            if step_tile != curent_tile {
                                for (z, other) in at(step_tile) {
                                    if other.material().is_solid() {
                                        // TODO: consider materials better
                                        let [other_x, other_y] = other.position();
                                        let volume2 = other.volume();
                                        if volume.collide(x, y, &volume2, *other_x, *other_y) {
                                            tracing::trace!(name="physics-step-translation-collide", p=?position, xy=?step_tile);

                                            let collision = collision::Collision(i.clone(), z);
                                            let collision = Event::Collision(collision);
                                            events.push(collision);

                                            // Do not keep this force by stoping this iteration
                                            continue 'forces;
                                        }
                                    }
                                }
                            }

                            // // Test new tile only when line on new tile
                            // if step_tile != curent_tile {
                            //     let Some(tile) = objects(step_tile) else {
                            //         // No tile means outside map
                            //         tracing::trace!(name="physics-step-translation-no-tile", p=?position, xy=?step_tile);
                            //         events.push(Event::NoTile);
                            //         continue 'forces;
                            //     };

                            //     // Move is finished if its a solid
                            //     if tile.material().is_solid() {
                            //         tracing::trace!(name="physics-step-translation-solid", p=?position, xy=?step_tile);

                            //         let collision = collision::Collision(object, tile);
                            //         events.push(Event::Collision(collision.clone()));

                            //         // Do not keep this force by stoping this iteration
                            //         continue 'forces;
                            //     }
                            // }

                            curent_tile = step_tile;
                            position = [step_x, step_y];
                            tracing::trace!(name="physics-step-translation-updated", p=?position, xy=?step_tile);
                        }
                        line::Step::Outside => {
                            tracing::trace!(name="physics-step-translation-no-tile", p=?position);
                            events.push(Event::NoTile);
                            continue 'forces;
                        }
                    }
                }
            }
        }

        forces.push(force.clone());
    }

    tracing::trace!(name="physics-step-finished", position=?position, forces=?forces, events=events.len());
    (position, forces, events)
}

#[cfg(test)]
mod tests {
    use crate::collision::Materials;

    use super::*;

    struct MyObject([f32; 2], Vec<Force>);

    impl Physic for MyObject {
        fn position(&self) -> &[f32; 2] {
            &self.0
        }
        fn forces(&self) -> &Vec<Force> {
            &self.1
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
        let laws = Laws::default()
            .tick_coeff(0.5)
            .bresenham_precision(100.)
            .pixels_per_meters(10.);
        let direction = [1.0, 0.0]; // South
        let speed = MetersSeconds(1.0);
        let force = Force::Translation(direction, speed);
        let object = MyObject([0.0, 0.0], vec![force]);
        let tiles = |_| Some(&MyTile(Materials::Traversable));

        // When
        let (new_position, _) = step(&laws, &object, tiles);

        // Then
        let expected_new_position = [5.0, 0.0];
        assert_eq!(new_position, expected_new_position);
    }

    #[test]
    fn test_unidirectional_translation_collision() {
        // Given
        let laws = Laws::default()
            .tick_coeff(0.5)
            .bresenham_precision(100.)
            .bresenham_step(250)
            .pixels_per_meters(10.);
        let direction = [1.0, 0.0]; // South
        let speed = MetersSeconds(100.0);
        let force = Force::Translation(direction, speed);
        let object = MyObject([0.0, 0.0], vec![force]);
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
        let laws = Laws::default()
            .tick_coeff(0.5)
            .bresenham_precision(100.)
            .pixels_per_meters(10.);
        let direction = [1.0, 0.0]; // South
        let speed = MetersSeconds(10.0);
        let force = Force::Translation(direction, speed);
        let object = MyObject([0.0, 0.0], vec![force]);
        let tiles = |_| Some(&MyTile(Materials::Traversable));

        // When
        let (new_position, _) = step(&laws, &object, tiles);

        // Then
        let expected_new_position = [50.0, 0.0];
        assert_eq!(new_position, expected_new_position);
    }

    #[test]
    fn test_unidirectional_translation_high_speed_collision() {
        // Given
        let laws = Laws::default()
            .tick_coeff(0.5)
            .bresenham_precision(100.)
            .bresenham_step(250)
            .pixels_per_meters(10.);
        let direction = [1.0, 0.0]; // South
        let speed = MetersSeconds(100.0);
        let force = Force::Translation(direction, speed);
        let object = MyObject([0.0, 0.0], vec![force]);
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
        let laws = Laws::default()
            .tick_coeff(0.5)
            .bresenham_precision(100.)
            .pixels_per_meters(10.);
        let direction = [1.0, 1.0]; // South
        let speed = MetersSeconds(1.0);
        let force = Force::Translation(direction, speed);
        let object = MyObject([0.0, 0.0], vec![force]);
        let tiles = |_| Some(&MyTile(Materials::Traversable));

        // When
        let (new_position, _) = step(&laws, &object, tiles);

        // Then
        let expected_new_position = [5.0, 5.0];
        assert_eq!(new_position, expected_new_position);
    }
}
