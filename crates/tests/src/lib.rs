#[cfg(test)]
mod test {
    use glam::Vec3;
    use oc_geo::tile::WorldTileIndex;
    use oc_physics::{
        Event, Force, Physic,
        collision::{Material, Materials},
        volume::Volume,
    };
    use oc_root::{WorldConfig, physics::MetersSeconds};
    use oc_world::tile::{Nature, Tile};
    use rstest::rstest;

    fn init_tracing() {
        let _ = tracing_subscriber::fmt()
            .with_test_writer()
            .with_env_filter("TRACE")
            .try_init();
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum ObjectsId {
        Tile(WorldTileIndex),
        Object(ObjectId),
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct ObjectId(usize);
    struct Object(Vec3, Vec<Force>);

    impl Physic for Object {
        fn position(&self, _: &WorldConfig) -> [f32; 3] {
            self.0.into()
        }

        fn forces(&self, _: &WorldConfig) -> &Vec<Force> {
            &self.1
        }

        fn volume(&self, ref_: [f32; 3], _: &WorldConfig) -> Volume {
            Volume::Point {
                x: ref_[0],
                y: ref_[1],
                z: ref_[2],
            }
        }
    }

    impl Material for Object {
        fn material(&self) -> Materials {
            todo!()
        }
    }

    #[rstest]
    // Case 1
    #[case(
        // Object at identical pos than tile, but without force
        [0., 0., 0.], vec![],
        // produce nothing
        ([0., 0., 0.], vec![], vec![])
    )]
    // Case 2
    #[case(
        // Object at identical pos than tile, with movement to the ground
        [0., 0., 0.], vec![Force::Translation([0., 0., -1.], MetersSeconds(1.))],
        // produce no collistion, because collisions tested on new tile hovering only
        ([0., 0., -5.], vec![Force::Translation([0., 0., -1.], MetersSeconds(1.))], vec![])
    )]
    // Case 2
    #[case(
        // Object at other pos than tile, with movement to the ground, in the tile
        [5.1, 5.1, 0.], vec![Force::Translation([-1., -1., -1.], MetersSeconds(1.))],
        // produce collistion
        ([2.6, 2.6, -2.5], vec![], vec![Event::Collision(ObjectsId::Object(ObjectId(0)), ObjectsId::Tile(WorldTileIndex(0)))])
    )]
    fn test_tile_collision_in_meters_zero(
        #[case] pos: [f32; 3],
        #[case] forces: Vec<Force>,
        #[case] expected: ([f32; 3], Vec<Force>, Vec<Event<ObjectsId>>),
    ) {
        init_tracing();

        // Given
        let w = WorldConfig::new(1000, 1000).geo_pixels_per_tile(5);

        let tile_i = WorldTileIndex(0);
        let tile = Tile {
            i: tile_i,
            nature: Nature::ShortGrass,
            z: 0,
        };

        let object_i = ObjectId(0);
        let object = Object(pos.into(), forces);

        // When
        let delta = 1.0;
        let result = oc_physics::step(
            &w,
            delta,
            (ObjectsId::Object(object_i), &object),
            |_| vec![(ObjectsId::Tile(tile_i), Box::new(&tile))],
            "tests",
        );

        assert_eq!(result, expected);

        // FIXME BS NOW
        // Il faut notion de z * ratio
        // tester avec des metres comme position
        // dbg!((position, forces, events));
    }
}
