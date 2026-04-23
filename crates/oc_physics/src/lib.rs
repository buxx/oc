use oc_root::{
    GEO_BRESENHAM_PRECISION, GEO_BRESENHAM_STEP, GEO_PIXELS_PER_METERS, GEO_PIXELS_PER_TILE,
    PHYSICS_COEFF_PER_TICK, physics::MetersSeconds,
};
use oc_utils::d2::Xy;
use rkyv::{Archive, Deserialize, Serialize};

use crate::{collision::Material, volume::Volume};

pub mod collision;
pub mod fx;
pub mod line;
pub mod reactive;
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

pub trait Physic: Material {
    // TODO: maby position should be `Geo` instead `Physics`...
    fn position(&self) -> [f32; 3];
    fn forces(&self) -> &Vec<Force>;
    fn volume(&self, ref_: [f32; 3]) -> Volume;
}

pub trait UpdatePhysic: Physic + Material {
    fn set_position(&mut self, value: [f32; 3]);
    fn push_force(&mut self, value: Force);
    fn remove_force(&mut self, value: &Force);
    fn set_volume(&self, value: Volume);
}

#[derive(Debug)]
pub struct Corps<I: Clone + std::fmt::Debug> {
    pub i: I,
    position: [f32; 3],
    forces: Vec<Force>,
    material: collision::Materials,
    volume: volume::Volume,
}

impl<I: Clone + std::fmt::Debug> Corps<I> {
    pub fn new(
        i: I,
        position: [f32; 3],
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
    fn position(&self) -> [f32; 3] {
        self.position
    }

    fn forces(&self) -> &Vec<Force> {
        &self.forces
    }

    fn volume(&self, ref_: [f32; 3]) -> Volume {
        self.volume.clone().with_ref(ref_)
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
    Translation([f32; 3], MetersSeconds),
}

#[derive(Debug, Clone)]
pub enum Event<T> {
    NoTile(T),
    Collision(T, T),
}

pub trait World<Z> {
    fn at(&self, xy: Xy) -> Vec<(Z, Box<dyn Physic>)>;
}

#[inline]
pub fn step<'a, I, O, F, Z>(
    laws: &Laws,
    object: (I, &'a O),
    at: F,
    origin: &str,
) -> ([f32; 3], Vec<Force>, Vec<Event<Z>>)
where
    I: Clone + Into<Z> + std::fmt::Debug,
    O: Physic,
    F: Fn(Xy) -> Vec<(Z, Box<&'a dyn Physic>)>,
    Z: std::fmt::Debug,
{
    let (i, object) = object;
    let mut events = vec![];
    let mut position = object.position().clone();
    let mut forces = vec![];
    tracing::trace!(name="physics-step-start", origin=origin, i=?i, p=?position, forces=?object.forces());

    'forces: for force in object.forces() {
        match force {
            Force::Translation(direction, speed) => {
                let speed = speed.0 * laws.tick_coeff;
                let pixels = speed * laws.pixels_per_meters as f32;
                let [x, y, z] = position;
                let (x_, y_, z_) = (
                    x + direction[0] * pixels,
                    y + direction[1] * pixels,
                    z + direction[2] * pixels,
                );

                tracing::trace!(
                    name = "physics-step-translation-start", origin=origin, i=?i,
                    x = x,
                    y = y,
                    z = z,
                    x_ = x_,
                    y_ = y_,
                    z_ = z_,
                    speed = speed
                );

                let mut curent_tile = Xy(
                    x as u64 / GEO_PIXELS_PER_TILE,
                    y as u64 / GEO_PIXELS_PER_TILE,
                );

                for step in line::Steps::new(laws, (x, y, z), (x_, y_, z_)) {
                    match step {
                        line::Step::First([step_x, step_y, step_z], step_tile)
                        | line::Step::Inside([step_x, step_y, step_z], step_tile)
                        | line::Step::Last([step_x, step_y, step_z], step_tile) => {
                            // Test new tile only when line on new tile
                            if step_tile != curent_tile {
                                let volume = object.volume([step_x, step_y, step_z]);
                                tracing::trace!(name="physics-step-translation-newtile", origin=origin, i=?i, p=?position, xy=?step_tile);

                                for (o, other) in at(step_tile) {
                                    // if other.material().is_solid() {
                                    let [other_x, other_y, other_z] = other.position();
                                    let volume2 = other.volume([other_x, other_y, other_z]);

                                    tracing::trace!(name="physics-step-translation-test-collide-with", origin=origin, i=?i, p=?position, xy=?step_tile, o=?o, volume=?volume, volume2=?volume2);
                                    if volume.collide(&volume2) {
                                        tracing::trace!(name="physics-step-translation-collide", origin=origin, i=?i, p=?position, xy=?step_tile);

                                        let left = i.clone().into();
                                        let collision = Event::Collision(left, o);
                                        events.push(collision);

                                        // Do not keep this force by stoping this iteration
                                        continue 'forces;
                                    }
                                    // }
                                }
                            }

                            curent_tile = step_tile;
                            position = [step_x, step_y, step_z];
                            tracing::trace!(name="physics-step-translation-updated", origin=origin, i=?i, p=?position, xy=?step_tile);
                        }
                        line::Step::Outside => {
                            tracing::trace!(name="physics-step-translation-no-tile", origin=origin, i=?i, p=?position);
                            events.push(Event::NoTile(i.clone().into()));
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
    use oc_geo::tile::TileXy;

    use crate::collision::Materials;

    use super::*;

    struct MyObject([f32; 3], Vec<Force>);
    #[derive(Debug, Clone)]
    struct MyObjectId;

    impl Physic for MyObject {
        fn position(&self) -> [f32; 3] {
            self.0
        }
        fn forces(&self) -> &Vec<Force> {
            &self.1
        }

        fn volume(&self, ref_: [f32; 3]) -> Volume {
            Volume::Point {
                x: ref_[0],
                y: ref_[1],
                z: ref_[2],
            }
        }
    }

    impl Material for MyObject {
        fn material(&self) -> Materials {
            Materials::Traversable
        }
    }

    struct MyTile(TileXy, Materials);

    impl Physic for MyTile {
        fn position(&self) -> [f32; 3] {
            self.0.into()
        }

        fn forces(&self) -> &Vec<Force> {
            static EMPTY: Vec<Force> = vec![];
            &EMPTY
        }

        fn volume(&self, ref_: [f32; 3]) -> Volume {
            Volume::Cube {
                x: ref_[0],
                y: ref_[1],
                z: ref_[2],
                width: GEO_PIXELS_PER_TILE as f32,
                height: GEO_PIXELS_PER_TILE as f32,
                depth: f32::MAX,
            }
        }
    }

    impl Material for MyTile {
        fn material(&self) -> Materials {
            self.1
        }
    }

    #[test]
    fn test_unidirectional_translation() {
        // Given
        let laws = Laws::default()
            .tick_coeff(0.5)
            .bresenham_precision(100.)
            .pixels_per_meters(10.);
        let direction = [1.0, 0.0, 0.0]; // South
        let speed = MetersSeconds(1.0);
        let force = Force::Translation(direction, speed);
        let object = MyObject([0.0, 0.0, 0.0], vec![force]);

        // When
        let (new_position, _, _): ([f32; 3], Vec<Force>, Vec<Event<MyObjectId>>) =
            step(&laws, (MyObjectId, &object), |_| vec![], "test");

        // Then
        let expected_new_position = [5.0, 0.0, 0.0];
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
        let direction = [1.0, 0.0, 0.0]; // South
        let speed = MetersSeconds(100.0);
        let force = Force::Translation(direction, speed);
        let object = MyObject([0.0, 0.0, 0.0], vec![force]);
        let my_traversable_tile = MyTile(TileXy(Xy(0, 0)), Materials::Solid);
        let my_traversable_tile: Box<&dyn Physic> = Box::new(&my_traversable_tile);
        let my_solid_tile = MyTile(TileXy(Xy(1, 0)), Materials::Solid);
        let my_solid_tile: Box<&dyn Physic> = Box::new(&my_solid_tile);
        let objects = |xy| {
            if xy == Xy(0, 0) {
                return vec![(MyObjectId, my_traversable_tile.clone())];
            } else {
                return vec![(MyObjectId, my_solid_tile.clone())];
            }
        };

        // When
        let (new_position, new_forces, _): ([f32; 3], Vec<Force>, Vec<Event<MyObjectId>>) =
            step(&laws, (MyObjectId, &object), objects, "test");

        // Then
        let expected_new_position = [2.5, 0.0, 0.0];
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
        let direction = [1.0, 0.0, 0.0]; // South
        let speed = MetersSeconds(10.0);
        let force = Force::Translation(direction, speed);
        let object = MyObject([0.0, 0.0, 0.0], vec![force]);

        // When
        let (new_position, _, _): ([f32; 3], Vec<Force>, Vec<Event<MyObjectId>>) =
            step(&laws, (MyObjectId, &object), |_| vec![], "test");

        // Then
        let expected_new_position = [50.0, 0.0, 0.0];
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
        let direction = [1.0, 0.0, 0.0]; // South
        let speed = MetersSeconds(100.0);
        let force = Force::Translation(direction, speed);
        let object = MyObject([0.0, 0.0, 0.0], vec![force]);
        let my_traversable_tile = MyTile(TileXy(Xy(0, 0)), Materials::Solid);
        let my_traversable_tile: Box<&dyn Physic> = Box::new(&my_traversable_tile);
        let my_solid_tile = MyTile(TileXy(Xy(1, 0)), Materials::Solid);
        let my_solid_tile: Box<&dyn Physic> = Box::new(&my_solid_tile);
        let objects = |xy| {
            if xy == Xy(0, 0) {
                return vec![(MyObjectId, my_traversable_tile.clone())];
            } else {
                return vec![(MyObjectId, my_solid_tile.clone())];
            }
        };

        // When
        let (new_position, new_forces, _): ([f32; 3], Vec<Force>, Vec<Event<MyObjectId>>) =
            step(&laws, (MyObjectId, &object), objects, "test");

        // Then
        let expected_new_position = [2.5, 0.0, 0.0];
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
        let direction = [1.0, 1.0, 0.0]; // South
        let speed = MetersSeconds(1.0);
        let force = Force::Translation(direction, speed);
        let object = MyObject([0.0, 0.0, 0.0], vec![force]);

        // When
        let (new_position, _, _): ([f32; 3], Vec<Force>, Vec<Event<MyObjectId>>) =
            step(&laws, (MyObjectId, &object), |_| vec![], "test");

        // Then
        let expected_new_position = [5.0, 5.0, 0.0];
        assert_eq!(new_position, expected_new_position);
    }
}
