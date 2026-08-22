use std::f32;

use oc_geo::{
    region::WorldRegionIndex,
    tile::{TileXy, WorldTileIndex},
};
use oc_mod::nature::Traversability;
use oc_mod::{Mod, nature::NatureIndex};
use oc_physics::{Force, IgnoreSide, Physic, collision::Material, volume::Volume};
use oc_root::{WcfgInto, WorldConfig, geo::WorldVec3, material::MaterialKind};
use oc_utils::d2::Direction;

use crate::{World, navmesh::Walls};
use derive_more::Constructor;
use rkyv::{Archive, Deserialize, Serialize};

const DEPTH: f32 = 10_000.;

#[derive(Debug, Clone, Archive, Deserialize, Serialize, PartialEq, Constructor)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct Tile {
    pub i: WorldTileIndex, // Should not be necessary, but oc_physics::step must take a reference ...
    pub nature: NatureIndex,
    pub z: u8,
    // Copy it from nature for performance consideration.
    // If it use too much RAM, consider read it through Mod
    pub allow: Traversability,
}

impl Tile {
    pub fn z_pixels(&self, w: &WorldConfig) -> f32 {
        self.z as f32 * w.geo_meters_per_z.0 * w.geo_pixels_per_meters
    }
}

pub trait AsTiles {
    fn as_tiles<'a>(&self, world: &'a World) -> Vec<(WorldTileIndex, &'a Tile)>;
}

impl AsTiles for WorldRegionIndex {
    fn as_tiles<'a>(&self, world: &'a World) -> Vec<(WorldTileIndex, &'a Tile)> {
        world.region_tiles(*self)
    }
}

pub trait IntoTiles {
    fn into_tiles(&self, world: &World) -> Vec<(WorldTileIndex, Tile)>;
}

impl IntoTiles for WorldRegionIndex {
    fn into_tiles(&self, world: &World) -> Vec<(WorldTileIndex, Tile)> {
        let tiles = self.as_tiles(world);
        tiles.into_iter().map(|(i, t)| (i, t.clone())).collect()
    }
}

impl Material for Tile {
    fn kind(&self) -> Option<oc_root::material::MaterialKind> {
        None
    }
}

impl Physic for Tile {
    fn position(&self, w: &WorldConfig) -> WorldVec3 {
        let xy: TileXy = self.i.into_(w);
        let point = xy.point(w);
        tracing::trace!(name="DEBUG", i=?self.i, xy=?xy, point=?point);
        WorldVec3::new(
            point[0],
            point[1],
            self.z as f32 * w.geo_meters_per_z.0 * w.geo_pixels_per_meters,
        )
    }

    fn forces(&self, _: &WorldConfig) -> &Vec<Force> {
        static EMPTY: Vec<Force> = vec![];
        &EMPTY
    }

    fn volumes(
        &self,
        ref_: WorldVec3,
        w: &WorldConfig,
        mod_: &Mod,
    ) -> Vec<(Volume, Traversability, Direction)> {
        tracing::trace!(name = "tile-volume", ref_ = ?ref_);
        let nature = mod_.nature(self.nature);
        let exceedance = nature.z.0 * w.geo_pixels_per_meters;

        vec![
            // Tile "ground". Always not traversable.
            (
                Volume::Cube {
                    x: ref_.x,
                    y: ref_.y,
                    z: -DEPTH,
                    width: w.geo_pixels_per_tile as f32,
                    height: w.geo_pixels_per_tile as f32,
                    depth: DEPTH + ref_.z,
                },
                Traversability::none(),
                Direction::NORTH,
            ),
            // Tile nature (hedge part for example)
            (
                Volume::Cube {
                    x: ref_.x,
                    y: ref_.y,
                    z: ref_.z,
                    width: w.geo_pixels_per_tile as f32,
                    height: w.geo_pixels_per_tile as f32,
                    depth: exceedance,
                },
                // TODO: perf test with reference ?
                nature.traversability.clone(),
                Direction::NORTH,
            ),
        ]
    }

    fn ignore_side(&self) -> IgnoreSide {
        IgnoreSide::None
    }

    fn side(&self) -> Option<oc_root::side::Side> {
        None
    }
}

impl Walls for Vec<Tile> {
    fn as_walls(&self, mod_: &Mod) -> Vec<bool> {
        self.iter()
            .map(|tile| {
                // FIXME: When vehicle, will need same but for vehicle
                mod_.nature(tile.nature)
                    .traversability
                    .deny(MaterialKind::Individual)
            })
            .collect::<Vec<bool>>()
    }
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use super::*;
    use oc_root::{
        WcfgFrom,
        physics::{Meters, MetersSeconds},
        side::Side,
    };

    struct MyObject(WorldVec3, Vec<Force>);
    #[derive(Debug, Clone, serde::Serialize, PartialEq)]
    struct MyObjectId(usize);

    impl Physic for MyObject {
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

        fn ignore_side(&self) -> IgnoreSide {
            IgnoreSide::None
        }

        fn side(&self) -> Option<Side> {
            None
        }
    }

    impl Material for MyObject {
        fn kind(&self) -> Option<MaterialKind> {
            Some(MaterialKind::Projectile)
        }
    }
    fn workspace_root() -> PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf()
    }

    #[test]
    fn test_collision_on_tile_zero_z() {
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
        let w = WorldConfig::new(10, 10, Meters(0.1))
            .physics_coeff_per_tick(1.0)
            .geo_pixels_per_meters(10.)
            .geo_pixels_per_tile(5);
        let delta = w.physics_coeff_per_tick;
        let from = glam::Vec3::new(0., 0., 10.);
        let to = glam::Vec3::new(10., 10., 0.);
        let direction = (to - from).normalize_or_zero();
        let direction = WorldVec3::new(direction.x, direction.y, direction.z);

        let speed = MetersSeconds(1000.0);
        let force = Force::Translation(direction, speed);
        // Use `Tile` because we want test volumes
        let tiles: Vec<Tile> = (0..(10 * 10))
            .map(|i| Tile::new(WorldTileIndex(i), NatureIndex(0), 0, Traversability::all()))
            .collect();
        // let my_tile: Box<&dyn Physic> = Box::new(&my_tile);
        let object = MyObject([0.0, 0.0, 10.0].into(), vec![force]);
        let objects = |xy| -> Vec<(MyObjectId, Box<&dyn Physic>)> {
            let tile = WorldTileIndex::from_(TileXy(xy), &w);
            vec![(MyObjectId(1), Box::new(&tiles[tile.0 as usize]))]
        };

        // When
        let (position, forces, events): (
            WorldVec3,
            Vec<Force>,
            Vec<oc_physics::Event<MyObjectId>>,
        ) = oc_physics::step(
            &w,
            &mod_,
            delta,
            (MyObjectId(0), &object),
            objects,
            0,
            "test",
        );

        // Then
        let expected_position: WorldVec3 = [10.0, 10.0, 0.0].into();
        assert_eq!(position, expected_position); // x axis hit first, at MySubject (x) 2.0 (position) + 2.0 (width)
        assert_eq!(forces, Vec::<Force>::new());
        assert_eq!(
            events,
            vec![oc_physics::Event::Collision(MyObjectId(0), MyObjectId(1))]
        );
    }
}
