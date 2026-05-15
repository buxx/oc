#[cfg(test)]
mod test {
    use glam::Vec3;
    use oc_geo::tile::WorldTileIndex;
    use oc_physics::{
        Force, Physic,
        collision::{Material, Materials},
        volume::Volume,
    };
    use oc_root::WorldConfig;
    use oc_world::tile::{Nature, Tile};

    #[derive(Debug, Clone)]
    enum ObjectsId {
        Tile(WorldTileIndex),
        Object(ObjectId),
    }

    #[derive(Debug, Clone)]
    struct ObjectId(usize);
    struct Object(Vec3);

    impl Physic for Object {
        fn position(&self, _: &WorldConfig) -> [f32; 3] {
            self.0.into()
        }

        fn forces(&self, _: &WorldConfig) -> &Vec<Force> {
            static EMPTY: Vec<Force> = vec![];
            &EMPTY
        }

        fn volume(&self, ref_: [f32; 3], _: &WorldConfig) -> Volume {
            Volume::Point {
                x: self.0.x + ref_[0],
                y: self.0.y + ref_[1],
                z: self.0.y + ref_[2],
            }
        }
    }

    impl Material for Object {
        fn material(&self) -> Materials {
            todo!()
        }
    }

    #[test]
    fn test_tile_collision_in_meters_zero() {
        // Given
        let w = WorldConfig::new(1000, 1000);

        let tile_i = WorldTileIndex(0);
        let tile = Tile {
            i: tile_i,
            nature: Nature::ShortGrass,
            z: 0,
        };

        let object_i = ObjectId(0);
        let object = Object(Vec3::new(0., 0., 0.));

        // When
        let delta = 1.0;
        let (position, forces, events) = oc_physics::step(
            &w,
            delta,
            (ObjectsId::Object(object_i), &object),
            |_| vec![(ObjectsId::Tile(tile_i), Box::new(&tile))],
            "tests",
        );

        // FIXME BS NOW
        // Il faut notion de z * ratio
        // tester avec des metres comme position
        dbg!((position, forces, events));
    }
}
