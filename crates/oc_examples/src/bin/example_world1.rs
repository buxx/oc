use std::path::PathBuf;

use anyhow::Context;
use oc_examples::{
    logging, run,
    snapshot::{EmptyGenerator, SnapshotBuilder},
};
use oc_geo::{
    region::RegionXy,
    tile::{TileXy, WorldTileIndex},
};
use oc_individual::{Gesture, Individual, Status, behavior::Behavior};
use oc_mod::Mod;
use oc_projectile::Projectile;
use oc_root::{WcfgFrom, WcfgInto, WorldConfig, physics::Meters};
use oc_world::{meta::Meta, tile::Tile};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    logging::setup_logging()?;

    let mod_ = Mod::load(&PathBuf::from("mods/std1"), None)?;
    let meta = Meta::from_file(&PathBuf::from("examples/world1/meta.toml"))?;
    let map_ = PathBuf::from("examples/world1");
    let map = oc_world::reader::MapReader::new(&map_);
    let map = map.context(format!("Read map {}", map_.display()))?;
    let w = WorldConfig::new(
        map.width().unwrap() as u64,
        map.height().unwrap() as u64,
        Meters(meta.geo_meters_per_z),
    );
    let projectiles = EmptyGenerator::<Projectile>::new();
    let snapshot = SnapshotBuilder::new(map, individuals, projectiles).build(w, &mod_)?;

    let example = run::Example::builder()
        .world(PathBuf::from("examples/world1"))
        .mod_(PathBuf::from("mods/std1"))
        .snapshot(snapshot);
    example.build().run()?;

    Ok(())
}

fn individuals(w: &WorldConfig, tiles: &Vec<Tile>) -> Vec<Individual> {
    (0..10)
        .map(|i| {
            let tile_i = WorldTileIndex(i as u64);
            let tile_xy = TileXy::from_(tile_i, w);
            let tile = &tiles[tile_i.0 as usize];

            let position = [
                ((tile_xy.0.0 * w.geo_pixels_per_tile) + w.geo_pixels_per_tile / 2) as f32,
                ((tile_xy.0.1 * w.geo_pixels_per_tile) + w.geo_pixels_per_tile / 2) as f32,
                tile.z as f32,
            ];
            let region: RegionXy = tile_xy.into_(w);
            Individual::new(
                position,
                tile_xy.into_(w),
                region.into_(w),
                Behavior::Idle,
                vec![],
                Status::Operational,
                Gesture::Idle,
            )
        })
        .collect()
}
