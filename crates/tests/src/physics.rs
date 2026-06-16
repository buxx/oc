#[cfg(test)]
mod test {
    use glam::Vec3;
    use oc_geo::tile::WorldTileIndex;
    use oc_mod::nature::Traversability;
    use oc_mod::{Mod, nature::NatureIndex};
    use oc_physics::{Event, Force, Physic, collision::Material, volume::Volume};
    use oc_root::{
        WorldConfig,
        physics::{Meters, MetersSeconds},
    };
    use oc_world::tile::Tile;
    use rstest::rstest;
    use serde::Serialize;
    use std::path::PathBuf;

    fn workspace_root() -> PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf()
    }

    fn init_tracing() {
        let _ = tracing_subscriber::fmt()
            .with_test_writer()
            .with_env_filter("ERROR")
            .try_init();
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize)]
    enum ObjectsId {
        Tile(WorldTileIndex),
        Object(ObjectId),
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize)]
    struct ObjectId(usize);
    struct Object(Vec3, Vec<Force>);

    impl Physic for Object {
        fn position(&self, _: &WorldConfig) -> [f32; 3] {
            self.0.into()
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

    impl Material for Object {}

    #[rstest]
    // Case 1
    #[case(
        // Object at identical pos than tile, but without force
        (0., 0., Meters(0.)), vec![],
        // produce nothing
        Meters(0.),
        ([0., 0., 0.], vec![], vec![])
    )]
    // Case 2
    #[case(
        // Object at identical pos than tile, with movement to the ground
        (0., 0., Meters(0.)), vec![Force::Translation([0., 0., -1.], MetersSeconds(1.))],
        // produce no collision, because collisions tested on new tile hovering only
        Meters(0.),
        ([0., 0., -5.], vec![Force::Translation([0., 0., -1.], MetersSeconds(1.))], vec![])
    )]
    // Case 3
    #[case(
        // Object at other pos than tile, with movement to the ground, in the tile
        (5.1, 5.1, Meters(0.)), vec![Force::Translation([-1., -1., -1.], MetersSeconds(1.))],
        // produce collision
        Meters(0.),
        ([2.6, 2.6, -2.5], vec![], vec![Event::Collision(ObjectsId::Object(ObjectId(0)), ObjectsId::Tile(WorldTileIndex(0)))])
    )]
    // Case 4
    #[case(
        // Incomming object a 10 meters
        (5.1, 5.1, Meters(10.)), vec![Force::Translation([-1., -1., 0.], MetersSeconds(1.))],
        // produce collision with a tile at 12 meters
        Meters(12.),
        ([2.6, 2.6, 50.0], vec![], vec![Event::Collision(ObjectsId::Object(ObjectId(0)), ObjectsId::Tile(WorldTileIndex(0)))])
    )]
    fn test_tile_collision_in_meters_zero(
        #[case] object_pos: (f32, f32, Meters),
        #[case] object_forces: Vec<Force>,
        #[case] tile_meters: Meters,
        #[case] expected: ([f32; 3], Vec<Force>, Vec<Event<ObjectsId>>),
    ) {
        init_tracing();

        // Given
        let mod_ = Mod::load(&workspace_root().join("mods/tests1"), None).unwrap();
        let geo_meters_per_z = Meters(0.1);
        let w = WorldConfig::new(1000, 1000, geo_meters_per_z)
            .geo_pixels_per_tile(5)
            .geo_pixels_per_meters(5.);

        let tile_i = WorldTileIndex(0);
        let tile_z = (tile_meters.0 / geo_meters_per_z.0) as u8;
        let tile = Tile {
            i: tile_i,
            nature: NatureIndex(0),
            z: tile_z,
            prohibe: Traversability::all(),
        };

        let object_i = ObjectId(0);
        let object_x = object_pos.0;
        let object_y = object_pos.1;
        let object_z = object_pos.2.0 * w.geo_pixels_per_meters;
        let object = Object(Vec3::new(object_x, object_y, object_z), object_forces);

        // When
        let delta = 1.0;
        let result = oc_physics::step(
            &w,
            &mod_,
            delta,
            (ObjectsId::Object(object_i), &object),
            |_| vec![(ObjectsId::Tile(tile_i), Box::new(&tile))],
            "tests",
        );

        assert_eq!(result, expected);
    }
}
