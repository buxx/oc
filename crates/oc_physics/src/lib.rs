use crate::{collision::Material, volume::Volume};
use line_drawing::Bresenham3d;
use oc_mod::{Mod, nature::Traversability};
use oc_root::{WcfgFrom, WorldConfig, physics::MetersSeconds};
use oc_utils::d2::Xy;
use rkyv::Archive;

pub mod collision;
pub mod corps;
pub mod fx;
pub mod reactive;
pub mod translation;
pub mod update;
pub mod volume;

pub trait Physic: Material {
    // TODO: maby position should be `Geo` instead `Physics`...
    fn position(&self, w: &WorldConfig) -> [f32; 3];
    fn forces(&self, w: &WorldConfig) -> &Vec<Force>;
    fn volumes(&self, ref_: [f32; 3], w: &WorldConfig, mod_: &Mod)
    -> Vec<(Volume, Traversability)>;
}

pub trait UpdatePhysic: Physic + Material {
    fn set_position(&mut self, value: [f32; 3]);
    fn push_force(&mut self, value: Force);
    fn remove_force(&mut self, value: &Force);
    fn set_volumes(&self, value: Vec<(Volume, Traversability)>);
}

// TODO: gravité
#[derive(Archive, rkyv::Deserialize, rkyv::Serialize, Debug, PartialEq, Clone)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub enum Force {
    Translation([f32; 3], MetersSeconds),
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub enum Event<T: serde::Serialize> {
    NoTile(T),
    Collision(T, T),
}

pub trait World<Z> {
    fn at(&self, xy: Xy) -> Vec<(Z, Box<dyn Physic>)>;
}

#[inline]
pub fn step<'a, I, O, F, Z>(
    w: &WorldConfig,
    mod_: &Mod,
    delta: f32,
    object: (I, &'a O),
    at: F,
    origin: &str,
) -> ([f32; 3], Vec<Force>, Vec<Event<Z>>)
where
    I: Clone + Into<Z> + std::fmt::Debug,
    O: Physic,
    F: Fn(Xy) -> Vec<(Z, Box<&'a dyn Physic>)>,
    Z: std::fmt::Debug + serde::Serialize + PartialEq,
{
    let (i, object) = object;
    let mut events = vec![];
    let mut position = object.position(w);
    let mut forces = vec![];
    let kind = object.kind();
    tracing::trace!(name="physics-step-start", origin=origin, i=?i, p=?position, forces=?object.forces(w));

    'forces: for force in object.forces(w) {
        match force {
            Force::Translation(direction, speed) => {
                let speed = speed.0 * delta;
                let pixels = speed * w.geo_pixels_per_meters;
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
                    speed = speed,
                    pixels = pixels,
                );

                let start = (x as isize, y as isize, z as isize);
                let end = (x_ as isize, y_ as isize, z_ as isize);
                let world_width = w.world_width_pixels as u64;
                let world_height = w.world_width_pixels as u64;
                let mut interupted = false;

                'pixels: for (pixel_x, pixel_y, pixel_z) in Bresenham3d::new(start, end) {
                    if pixel_x < 0
                        || pixel_y < 0
                        || pixel_x >= world_width as isize
                        || pixel_y >= world_height as isize
                    {
                        tracing::trace!(name="physics-step-translation-outside", origin=origin, i=?i, pixel=?(pixel_x, pixel_y, pixel_z));
                        // Outside world
                        interupted = true;
                        // FIXME BS NOW: tester si on a pas regressé sur la dispoarition de l'objet (sortie de carte)
                        break 'pixels;
                    }

                    let pixel = [pixel_x as f32, pixel_y as f32, pixel_z as f32];
                    let xy = Xy::from_((pixel_x, pixel_y), w);
                    position = [pixel_x as f32, pixel_y as f32, pixel_z as f32];

                    tracing::trace!(name="physics-step-translation-line-pixel", origin=origin, i=?i, pixel=?pixel, xy=?xy);
                    let volumes = object.volumes(pixel, w, mod_);

                    for (o, other) in at(xy) {
                        // Do not test collision with itself (it is possible than `at` return it)
                        // NOTE: Maybe not the most optimized thing ?
                        if o == i.clone().into() {
                            continue;
                        }

                        tracing::trace!(name="physics-step-translation-other", origin=origin, i=?i, o=?o);

                        let [other_x, other_y, other_z] = other.position(w);
                        let position2 = [other_x, other_y, other_z];

                        for (volume1, traversability1) in &volumes {
                            let volumes2 = other.volumes(position2, w, mod_);
                            'other_volumes: for (volume2, traversability2) in volumes2 {
                                // Test volumes collision only if object own a kind and other own too, and prohibe it on its tile
                                tracing::trace!(name="physics-step-translation-prohibe-test", origin=origin, i=?i, traversability1=?traversability1, traversability2=?traversability2);
                                if kind.map(|kind| traversability2.allow(kind)).unwrap_or(true) {
                                    tracing::trace!(name="physics-step-translation-prohibe-allow", origin=origin, i=?i);
                                    continue 'other_volumes;
                                }

                                tracing::trace!(name="physics-step-translation-test-collide-with", origin=origin, i=?i, p=?position, xy=?xy, o=?o, op=?[other_x, other_y, other_z], volume1=?volume1, volume2=?volume2);
                                if volume1.collide(&volume2) {
                                    tracing::trace!(name="physics-step-translation-collide", origin=origin, i=?i, p=?position, xy=?xy);

                                    let left = i.clone().into();
                                    let collision = Event::Collision(left, o);
                                    events.push(collision);

                                    // Do not keep this force by stopping this iteration
                                    position = pixel;
                                    continue 'forces;
                                }
                            }
                        }
                    }

                    tracing::trace!(name="physics-step-translation-updated", origin=origin, i=?i, p=?position);
                }
                if !interupted {
                    // If not interupted, position is now end of translation (bresenham3d accept only usize)
                    position = [x_, y_, z_];
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
    use std::path::PathBuf;

    use oc_geo::tile::TileXy;
    use oc_root::{WcfgInto, material::MaterialKind, physics::Meters};

    use super::*;

    fn workspace_root() -> PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf()
    }

    struct MyObject([f32; 3], Vec<Force>);
    #[derive(Debug, Clone, serde::Serialize, PartialEq)]
    struct MyObjectId(usize);

    impl Physic for MyObject {
        fn position(&self, _: &WorldConfig) -> [f32; 3] {
            self.0
        }
        fn forces(&self, _: &WorldConfig) -> &Vec<Force> {
            &self.1
        }

        fn volumes(
            &self,
            ref_: [f32; 3],
            _: &WorldConfig,
            _: &Mod,
        ) -> Vec<(Volume, Traversability)> {
            vec![(
                Volume::Point {
                    x: ref_[0],
                    y: ref_[1],
                    z: ref_[2],
                },
                Traversability::all(),
            )]
        }
    }

    impl Material for MyObject {
        fn kind(&self) -> Option<MaterialKind> {
            Some(MaterialKind::Projectile)
        }
    }

    struct MyTile(TileXy, Traversability);

    impl Physic for MyTile {
        fn position(&self, w: &WorldConfig) -> [f32; 3] {
            self.0.into_(w)
        }

        fn forces(&self, _: &WorldConfig) -> &Vec<Force> {
            static EMPTY: Vec<Force> = vec![];
            &EMPTY
        }

        fn volumes(
            &self,
            ref_: [f32; 3],
            w: &WorldConfig,
            _: &Mod,
        ) -> Vec<(Volume, Traversability)> {
            vec![(
                Volume::Cube {
                    x: ref_[0],
                    y: ref_[1],
                    z: ref_[2],
                    width: w.geo_pixels_per_tile as f32,
                    height: w.geo_pixels_per_tile as f32,
                    depth: f32::MAX,
                },
                self.1.clone(),
            )]
        }
    }

    impl Material for MyTile {
        fn kind(&self) -> Option<MaterialKind> {
            None
        }
    }

    struct MyIndividual([f32; 3]);

    impl Physic for MyIndividual {
        fn position(&self, _w: &WorldConfig) -> [f32; 3] {
            self.0
        }

        fn forces(&self, _: &WorldConfig) -> &Vec<Force> {
            static EMPTY: Vec<Force> = vec![];
            &EMPTY
        }

        fn volumes(
            &self,
            ref_: [f32; 3],
            _w: &WorldConfig,
            _: &Mod,
        ) -> Vec<(Volume, Traversability)> {
            vec![(
                Volume::Cube {
                    x: ref_[0],
                    y: ref_[1],
                    z: ref_[2],
                    width: 2.0,
                    height: 2.0,
                    depth: 10.,
                },
                Traversability::none(),
            )]
        }
    }

    impl Material for MyIndividual {
        fn kind(&self) -> Option<MaterialKind> {
            None
        }
    }

    #[test]
    fn test_unidirectional_translation_x() {
        // Given
        let mod_ = Mod::load(&workspace_root().join("mods/tests1"), None).unwrap();
        let w = WorldConfig::new(1000, 1000, Meters(0.1))
            .physics_coeff_per_tick(0.5)
            .geo_pixels_per_meters(10.);
        let delta = w.physics_coeff_per_tick;
        let direction = [1.0, 0.0, 0.0];
        let speed = MetersSeconds(1.0);
        let force = Force::Translation(direction, speed);
        let object = MyObject([0.0, 0.0, 0.0], vec![force]);

        // When
        let (new_position, _, _): ([f32; 3], Vec<Force>, Vec<Event<MyObjectId>>) = step(
            &w,
            &mod_,
            delta,
            (MyObjectId(0), &object),
            |_| vec![],
            "test",
        );

        // Then
        let expected_new_position = [5.0, 0.0, 0.0];
        assert_eq!(new_position, expected_new_position);
    }

    #[test]
    fn test_unidirectional_translation_outside() {
        // Given
        let mod_ = Mod::load(&workspace_root().join("mods/tests1"), None).unwrap();
        let w = WorldConfig::new(10, 10, Meters(0.1))
            .physics_coeff_per_tick(1.0)
            .geo_pixels_per_meters(10.);
        let delta = w.physics_coeff_per_tick;
        let direction = [1.0, 0.0, 0.0];
        let speed = MetersSeconds(100.0);
        let force = Force::Translation(direction, speed);
        let object = MyObject([0.0, 0.0, 0.0], vec![force]);

        // When
        let (new_position, _, events): ([f32; 3], Vec<Force>, Vec<Event<MyObjectId>>) = step(
            &w,
            &mod_,
            delta,
            (MyObjectId(0), &object),
            |_| vec![],
            "test",
        );

        // Then
        let expected_new_position = [49.0, 0.0, 0.0];
        assert_eq!(new_position, expected_new_position);
        // FIXME BS NOW: remplacer le système pour savoir quand ça sort de l'écran
        assert_eq!(events, vec![]);
    }

    #[test]
    fn test_unidirectional_translation_multisteps() {
        // Given
        let mod_ = Mod::load(&workspace_root().join("mods/tests1"), None).unwrap();
        let w = WorldConfig::new(1000, 1000, Meters(0.1))
            .physics_coeff_per_tick(1.0)
            .geo_pixels_per_meters(10.);
        let delta = w.physics_coeff_per_tick;
        let direction = [1.0, 0.0, 0.0]; // South
        let speed = MetersSeconds(0.01); // 1% of 10 pixels = 0.1 pixel
        let force = Force::Translation(direction, speed);
        let object = MyObject([0.0, 0.0, 0.0], vec![force]);

        // When
        let (new_position, _, _): ([f32; 3], Vec<Force>, Vec<Event<MyObjectId>>) = step(
            &w,
            &mod_,
            delta,
            (MyObjectId(0), &object),
            |_| vec![],
            "test",
        );

        // Then
        let expected_new_x = "0.1"; // step must complete bresenham pixel (which are isize) with end (force) position
        assert_eq!(&format!("{:.01}", new_position[0]), expected_new_x);
    }

    #[test]
    fn test_unidirectional_translation_collision() {
        // Given
        let mod_ = Mod::load(&workspace_root().join("mods/tests1"), None).unwrap();
        let w = WorldConfig::new(1000, 1000, Meters(0.1))
            .physics_coeff_per_tick(0.5)
            .geo_pixels_per_meters(10.);
        let delta = w.physics_coeff_per_tick;
        let direction = [1.0, 0.0, 0.0]; // South
        let speed = MetersSeconds(100.0);
        let force = Force::Translation(direction, speed);
        let object = MyObject([0.0, 0.0, 0.0], vec![force]);
        let my_traversable_tile = MyTile(TileXy(Xy(0, 0)), Traversability::all());
        let my_traversable_tile: Box<&dyn Physic> = Box::new(&my_traversable_tile);
        let my_solid_tile = MyTile(TileXy(Xy(1, 0)), Traversability::none());
        let my_solid_tile: Box<&dyn Physic> = Box::new(&my_solid_tile);
        let objects = |xy| {
            if xy == Xy(0, 0) {
                return vec![(MyObjectId(1), my_traversable_tile.clone())];
            } else {
                return vec![(MyObjectId(2), my_solid_tile.clone())];
            }
        };

        // When
        let (new_position, new_forces, _): ([f32; 3], Vec<Force>, Vec<Event<MyObjectId>>) =
            step(&w, &mod_, delta, (MyObjectId(0), &object), objects, "test");

        // Then
        let expected_new_position = [5.0, 0.0, 0.0];
        let expected_new_forces: Vec<Force> = vec![];
        assert_eq!(new_position, expected_new_position);
        assert_eq!(new_forces, expected_new_forces);
    }

    #[test]
    fn test_unidirectional_translation_high_speed() {
        // Given
        let mod_ = Mod::load(&workspace_root().join("mods/tests1"), None).unwrap();
        let w = WorldConfig::new(1000, 1000, Meters(0.1))
            .physics_coeff_per_tick(0.5)
            .geo_pixels_per_meters(10.);
        let delta = w.physics_coeff_per_tick;
        let direction = [1.0, 0.0, 0.0]; // South
        let speed = MetersSeconds(10.0);
        let force = Force::Translation(direction, speed);
        let object = MyObject([0.0, 0.0, 0.0], vec![force]);

        // When
        let (new_position, _, _): ([f32; 3], Vec<Force>, Vec<Event<MyObjectId>>) = step(
            &w,
            &mod_,
            delta,
            (MyObjectId(0), &object),
            |_| vec![],
            "test",
        );

        // Then
        let expected_new_position = [50.0, 0.0, 0.0];
        assert_eq!(new_position, expected_new_position);
    }

    #[test]
    fn test_unidirectional_translation_high_speed_collision() {
        // Given
        let mod_ = Mod::load(&workspace_root().join("mods/tests1"), None).unwrap();
        let w = WorldConfig::new(1000, 1000, Meters(0.1))
            .physics_coeff_per_tick(0.5)
            .geo_pixels_per_meters(10.);
        let delta = w.physics_coeff_per_tick;
        let direction = [1.0, 0.0, 0.0]; // South
        let speed = MetersSeconds(100.0);
        let force = Force::Translation(direction, speed);
        let object = MyObject([0.0, 0.0, 0.0], vec![force]);
        let my_traversable_tile = MyTile(TileXy(Xy(0, 0)), Traversability::all());
        let my_traversable_tile: Box<&dyn Physic> = Box::new(&my_traversable_tile);
        let my_solid_tile = MyTile(TileXy(Xy(1, 0)), Traversability::none());
        let my_solid_tile: Box<&dyn Physic> = Box::new(&my_solid_tile);
        let objects = |xy| {
            if xy == Xy(0, 0) {
                return vec![(MyObjectId(1), my_traversable_tile.clone())];
            } else {
                return vec![(MyObjectId(2), my_solid_tile.clone())];
            }
        };

        // When
        let (new_position, new_forces, _): ([f32; 3], Vec<Force>, Vec<Event<MyObjectId>>) =
            step(&w, &mod_, delta, (MyObjectId(0), &object), objects, "test");

        // Then
        let expected_new_position = [5., 0.0, 0.0];
        let expected_new_forces: Vec<Force> = vec![];
        assert_eq!(new_position, expected_new_position);
        assert_eq!(new_forces, expected_new_forces);
    }

    #[test]
    fn test_bidirectional_translation() {
        // Given
        let mod_ = Mod::load(&workspace_root().join("mods/tests1"), None).unwrap();
        let w = WorldConfig::new(1000, 1000, Meters(0.1))
            .physics_coeff_per_tick(0.5)
            .geo_pixels_per_meters(10.);
        let delta = w.physics_coeff_per_tick;
        let direction = [1.0, 1.0, 0.0]; // South
        let speed = MetersSeconds(1.0);
        let force = Force::Translation(direction, speed);
        let object = MyObject([0.0, 0.0, 0.0], vec![force]);

        // When
        let (new_position, _, _): ([f32; 3], Vec<Force>, Vec<Event<MyObjectId>>) = step(
            &w,
            &mod_,
            delta,
            (MyObjectId(0), &object),
            |_| vec![],
            "test",
        );

        // Then
        let expected_new_position = [5.0, 5.0, 0.0];
        assert_eq!(new_position, expected_new_position);
    }

    #[test]
    fn test_collision_with_volume_on_same_tile_centered() {
        // Given
        let mod_ = Mod::load(&workspace_root().join("mods/tests1"), None).unwrap();
        let w = WorldConfig::new(1000, 1000, Meters(0.1))
            .physics_coeff_per_tick(0.5)
            .geo_pixels_per_meters(10.)
            .geo_pixels_per_tile(5);
        let delta = w.physics_coeff_per_tick;
        let direction = [-1.0, 0.0, 0.0]; // West
        let speed = MetersSeconds(1.0);
        let force = Force::Translation(direction, speed);
        let object = MyObject([7.0, 2.0, 5.0], vec![force]);
        let individual = MyIndividual([2.0, 2.0, 0.0]); // MyIndividual volume is 2 px !
        let individual: Box<&dyn Physic> = Box::new(&individual);

        let objects = |xy| {
            // Expect collision when on tile 0,0
            if xy == Xy(0, 0) {
                vec![(MyObjectId(1), individual.clone())]
            } else {
                vec![]
            }
        };

        // When
        let (position, forces, events): ([f32; 3], Vec<Force>, Vec<Event<MyObjectId>>) =
            step(&w, &mod_, delta, (MyObjectId(0), &object), objects, "test");

        // Then
        assert_eq!(position, [4.0, 2.0, 5.0]); // x axis hit first, at MySubject (x) 2.0 (position) + 2.0 (width)
        assert_eq!(forces, Vec::<Force>::new());
        assert_eq!(events, vec![Event::Collision(MyObjectId(0), MyObjectId(1))]);
    }

    #[test]
    fn test_collision_with_volume_on_same_tile_decal() {
        // Given
        let mod_ = Mod::load(&workspace_root().join("mods/tests1"), None).unwrap();
        let w = WorldConfig::new(1000, 1000, Meters(0.1))
            .physics_coeff_per_tick(0.5)
            .geo_pixels_per_meters(10.)
            .geo_pixels_per_tile(5);
        let delta = w.physics_coeff_per_tick;
        let direction = [-1.0, 0.0, 0.0]; // West
        let speed = MetersSeconds(1.0);
        let force = Force::Translation(direction, speed);
        let object = MyObject([7.5, 2.5, 5.0], vec![force]); // Tile 1 (.5) on x; tile 0 (0.5) on y
        let individual = MyIndividual([1.0, 1.0, 0.0]); // MyIndividual volume size (see MyIndividual Physics impl) should be impacted
        let individual: Box<&dyn Physic> = Box::new(&individual);

        let objects = |xy| {
            // Expect collision when on tile 0,0
            if xy == Xy(0, 0) {
                vec![(MyObjectId(1), individual.clone())]
            } else {
                vec![]
            }
        };

        // When
        let (position, forces, events): ([f32; 3], Vec<Force>, Vec<Event<MyObjectId>>) =
            step(&w, &mod_, delta, (MyObjectId(0), &object), objects, "test");

        // Then
        assert_eq!(position, [3.0, 2.0, 5.0]);
        assert_eq!(forces, Vec::<Force>::new());
        assert_eq!(events, vec![Event::Collision(MyObjectId(0), MyObjectId(1))]);
    }

    #[test]
    fn test_collision_with_volume_on_near_tile() {}
}
