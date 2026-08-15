#[cfg(test)]
mod test {
    use oc_geo::tile::WorldTileIndex;
    use oc_mod::nature::Traversability;
    use oc_mod::{Mod, nature::NatureIndex};
    use oc_physics::{Event, Force, Physic, collision::Material, volume::Volume};
    #[cfg(test)]
    use oc_root::geo::WorldVec3;
    use oc_root::{
        WorldConfig,
        physics::{Meters, MetersSeconds},
    };
    use oc_utils::d2::{Direction, Xy};
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

    #[derive(Debug, Clone, PartialEq, Eq, Serialize)]
    enum ObjectsId {
        Tile(WorldTileIndex),
        Object(ObjectId),
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize)]
    struct ObjectId(usize);
    struct Object(WorldVec3, Vec<Force>);

    impl Physic for Object {
        fn position(&self, _: &WorldConfig) -> WorldVec3 {
            self.0
        }

        fn forces(&self, _: &WorldConfig) -> &Vec<Force> {
            &self.1
        }

        fn volumes(
            &self,
            ref_: WorldVec3,
            _: &WorldConfig,
            _: &Mod,
        ) -> Vec<(Volume, Traversability, Direction)> {
            vec![(
                Volume::Point {
                    x: ref_.x,
                    y: ref_.y,
                    z: ref_.z,
                },
                Traversability::all(),
                Direction::NORTH,
            )]
        }
    }

    impl Material for Object {
        fn kind(&self) -> Option<oc_root::material::MaterialKind> {
            Some(oc_root::material::MaterialKind::Projectile)
        }
    }

    #[rstest]
    // Case 1
    #[case(
        // Object at identical pos than tile, but without force
        (0., 0., Meters(0.)), vec![],
        // produce nothing
        Meters(0.),
        ([0., 0., 0.].into(), vec![], vec![])
    )]
    // Case 2
    #[case(
        // Object at other pos than tile, with movement to the ground, in the tile
        (5.1, 5.1, Meters(0.)), vec![Force::Translation([-1., -1., -1.].into(), MetersSeconds(1.))],
        // produce collision
        Meters(0.),
        ([4.0, 4.0, -1.0].into(), vec![], vec![Event::Collision(ObjectsId::Object(ObjectId(0)), ObjectsId::Tile(WorldTileIndex(0)))])
    )]
    // Case 3
    #[case(
        // Incoming object at 10 meters
        (5.1, 5.1, Meters(10.)), vec![Force::Translation([-1., -1., 0.].into(), MetersSeconds(1.))],
        // produce collision with a tile at 12 meters
        Meters(12.),
        ([4.0, 4.0, 50.0].into(), vec![], vec![Event::Collision(ObjectsId::Object(ObjectId(0)), ObjectsId::Tile(WorldTileIndex(0)))])
    )]
    fn test_tile_collision_in_meters_zero(
        #[case] object_pos: (f32, f32, Meters),
        #[case] object_forces: Vec<Force>,
        #[case] tile_meters: Meters,
        #[case] expected: (WorldVec3, Vec<Force>, Vec<Event<ObjectsId>>),
    ) {
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
            allow: Traversability::all(),
        };

        let object_i = ObjectId(0);
        let object_x = object_pos.0;
        let object_y = object_pos.1;
        let object_z = object_pos.2.0 * w.geo_pixels_per_meters;
        let object = Object(WorldVec3::new(object_x, object_y, object_z), object_forces);
        let tile: Box<&dyn Physic> = Box::new(&tile);

        let objects = |xy| {
            if xy == Xy(0, 0) {
                vec![(ObjectsId::Tile(tile_i), tile.clone())]
            } else {
                vec![]
            }
        };

        // When
        let delta = 1.0;
        let result = oc_physics::step(
            &w,
            &mod_,
            delta,
            (ObjectsId::Object(object_i), &object),
            objects,
            "tests",
        );

        assert_eq!(result, expected);
    }
}
