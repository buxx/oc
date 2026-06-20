use oc_mod::{Mod, nature::Traversability};
use oc_root::{WorldConfig, material::MaterialKind, physics::MetersSeconds};
use oc_utils::d2::Xy;
use rkyv::Archive;

use crate::{collision::Material, volume::Volume};

pub mod collision;
pub mod fx;
pub mod line;
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

#[derive(Debug)]
pub struct Corps<I: Clone + std::fmt::Debug> {
    pub i: I,
    position: [f32; 3],
    forces: Vec<Force>,
    material: Option<MaterialKind>,
    volumes: Vec<(volume::Volume, Traversability)>,
}

impl<I: Clone + std::fmt::Debug> Corps<I> {
    pub fn new(
        i: I,
        position: [f32; 3],
        forces: Vec<Force>,
        material: Option<MaterialKind>,
        volumes: Vec<(volume::Volume, Traversability)>,
    ) -> Self {
        Self {
            i,
            position,
            forces,
            material,
            volumes,
        }
    }
}

impl<I: Clone + std::fmt::Debug> Physic for Corps<I> {
    fn position(&self, _: &WorldConfig) -> [f32; 3] {
        self.position
    }

    fn forces(&self, _: &WorldConfig) -> &Vec<Force> {
        &self.forces
    }

    fn volumes(
        &self,
        ref_: [f32; 3],
        _: &WorldConfig,
        _mod_: &Mod,
    ) -> Vec<(Volume, Traversability)> {
        self.volumes
            .clone()
            .into_iter()
            .map(|(v, t)| (v.with_ref(ref_), t))
            .collect()
    }
}

impl<I: Clone + std::fmt::Debug> collision::Material for Corps<I> {
    fn kind(&self) -> Option<MaterialKind> {
        self.material
    }
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
    Z: std::fmt::Debug + serde::Serialize,
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
                    speed = speed
                );

                let mut curent_tile = Xy(
                    x as u64 / w.geo_pixels_per_tile,
                    y as u64 / w.geo_pixels_per_tile,
                );

                for step in line::Steps::new(
                    w.world_width_pixels,
                    w.world_height_pixels,
                    w.geo_bresenham_precision,
                    w.geo_bresenham_step,
                    w.geo_pixels_per_tile,
                    (x, y, z),
                    (x_, y_, z_),
                ) {
                    match step {
                        line::Step::First([step_x, step_y, step_z], step_tile)
                        | line::Step::Inside([step_x, step_y, step_z], step_tile)
                        | line::Step::Last([step_x, step_y, step_z], step_tile) => {
                            position = [step_x, step_y, step_z];

                            // FIXME BS NOW: how individual can be shot if we test it only on tile change ?!
                            // TODO: maybe test at each pixel ? (but perf ...)
                            // Test objects only when line on new tile
                            if step_tile != curent_tile {
                                curent_tile = step_tile;
                                let volumes = object.volumes([step_x, step_y, step_z], w, mod_);
                                tracing::trace!(name="physics-step-translation-newtile", origin=origin, i=?i, p=?position, xy=?step_tile);

                                for (o, other) in at(step_tile) {
                                    let [other_x, other_y, other_z] = other.position(w);
                                    let position2 = [other_x, other_y, other_z];

                                    for (volume1, traversability1) in &volumes {
                                        let volumes2 = other.volumes(position2, w, mod_);
                                        for (volume2, traversability2) in volumes2 {
                                            // Test volumes collision only if object own a kind and other own too, and prohibe it on its tile
                                            tracing::trace!(name="physics-step-translation-prohibe-test", origin=origin, i=?i, traversability1=?traversability1, traversability2=?traversability2);
                                            if kind
                                                .map(|kind| traversability2.allow(kind))
                                                .unwrap_or(true)
                                            {
                                                tracing::trace!(name="physics-step-translation-prohibe-allow", origin=origin, i=?i);
                                                continue;
                                            }

                                            tracing::trace!(name="physics-step-translation-test-collide-with", origin=origin, i=?i, p=?position, xy=?step_tile, o=?o, op=?[other_x, other_y, other_z], volume1=?volume1, volume2=?volume2);
                                            if volume1.collide(&volume2) {
                                                tracing::trace!(name="physics-step-translation-collide", origin=origin, i=?i, p=?position, xy=?step_tile);

                                                let left = i.clone().into();
                                                let collision = Event::Collision(left, o);
                                                events.push(collision);

                                                // Do not keep this force by stoping this iteration
                                                continue 'forces;
                                            }
                                        }
                                    }
                                }
                            } else {
                                curent_tile = step_tile;
                            }

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
    use std::path::PathBuf;

    use oc_geo::tile::TileXy;
    use oc_root::{WcfgInto, physics::Meters};

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
    #[derive(Debug, Clone, serde::Serialize)]
    struct MyObjectId;

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

    #[test]
    fn test_unidirectional_translation() {
        // Given
        let mod_ = Mod::load(&workspace_root().join("mods/tests1"), None).unwrap();
        let w = WorldConfig::new(1000, 1000, Meters(0.1))
            .physics_coeff_per_tick(0.5)
            .geo_bresenham_precision(100.)
            .geo_pixels_per_meters(10.);
        let delta = w.physics_coeff_per_tick;
        let direction = [1.0, 0.0, 0.0]; // South
        let speed = MetersSeconds(1.0);
        let force = Force::Translation(direction, speed);
        let object = MyObject([0.0, 0.0, 0.0], vec![force]);

        // When
        let (new_position, _, _): ([f32; 3], Vec<Force>, Vec<Event<MyObjectId>>) =
            step(&w, &mod_, delta, (MyObjectId, &object), |_| vec![], "test");

        // Then
        let expected_new_position = [5.0, 0.0, 0.0];
        assert_eq!(new_position, expected_new_position);
    }

    #[test]
    fn test_unidirectional_translation_collision() {
        // Given
        let mod_ = Mod::load(&workspace_root().join("mods/tests1"), None).unwrap();
        let w = WorldConfig::new(1000, 1000, Meters(0.1))
            .physics_coeff_per_tick(0.5)
            .geo_bresenham_precision(100.)
            .geo_bresenham_step(250)
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
                return vec![(MyObjectId, my_traversable_tile.clone())];
            } else {
                return vec![(MyObjectId, my_solid_tile.clone())];
            }
        };

        // When
        let (new_position, new_forces, _): ([f32; 3], Vec<Force>, Vec<Event<MyObjectId>>) =
            step(&w, &mod_, delta, (MyObjectId, &object), objects, "test");

        // Then
        let expected_new_position = [5.01, 0.0, 0.0];
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
            .geo_bresenham_precision(100.)
            .geo_pixels_per_meters(10.);
        let delta = w.physics_coeff_per_tick;
        let direction = [1.0, 0.0, 0.0]; // South
        let speed = MetersSeconds(10.0);
        let force = Force::Translation(direction, speed);
        let object = MyObject([0.0, 0.0, 0.0], vec![force]);

        // When
        let (new_position, _, _): ([f32; 3], Vec<Force>, Vec<Event<MyObjectId>>) =
            step(&w, &mod_, delta, (MyObjectId, &object), |_| vec![], "test");

        // Then
        let expected_new_position = [50.0, 0.0, 0.0];
        assert_eq!(new_position, expected_new_position);
    }

    #[test]
    fn test_unidirectional_translation_high_speed_collision() {
        tracing_subscriber::fmt()
            .with_target(false)
            .with_env_filter(
                tracing_subscriber::EnvFilter::builder()
                    .with_default_directive(tracing::level_filters::LevelFilter::TRACE.into())
                    .from_env()
                    .unwrap(),
            )
            .init();

        // Given
        let mod_ = Mod::load(&workspace_root().join("mods/tests1"), None).unwrap();
        let w = WorldConfig::new(1000, 1000, Meters(0.1))
            .physics_coeff_per_tick(0.5)
            .geo_bresenham_precision(100.)
            .geo_bresenham_step(250)
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
                return vec![(MyObjectId, my_traversable_tile.clone())];
            } else {
                return vec![(MyObjectId, my_solid_tile.clone())];
            }
        };

        // When
        let (new_position, new_forces, _): ([f32; 3], Vec<Force>, Vec<Event<MyObjectId>>) =
            step(&w, &mod_, delta, (MyObjectId, &object), objects, "test");

        // Then
        let expected_new_position = [5.01, 0.0, 0.0];
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
            .geo_bresenham_precision(100.)
            .geo_pixels_per_meters(10.);
        let delta = w.physics_coeff_per_tick;
        let direction = [1.0, 1.0, 0.0]; // South
        let speed = MetersSeconds(1.0);
        let force = Force::Translation(direction, speed);
        let object = MyObject([0.0, 0.0, 0.0], vec![force]);

        // When
        let (new_position, _, _): ([f32; 3], Vec<Force>, Vec<Event<MyObjectId>>) =
            step(&w, &mod_, delta, (MyObjectId, &object), |_| vec![], "test");

        // Then
        let expected_new_position = [5.0, 5.0, 0.0];
        assert_eq!(new_position, expected_new_position);
    }
}
