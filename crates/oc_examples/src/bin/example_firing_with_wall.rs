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
use oc_individual::{Individual, behavior::Behavior};
use oc_projectile::Projectile;
use oc_root::{WcfgFrom, WcfgInto, WorldConfig};
use oc_world::tile::Tile;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    logging::setup_logging()?;

    let map_ = PathBuf::from("examples/minidblue");
    let map = oc_world::reader::MapReader::new(&map_);
    let map = map.context(format!("Read map {}", map_.display()))?;
    let w = WorldConfig::new(map.width().unwrap() as u64, map.height().unwrap() as u64);
    let projectiles = EmptyGenerator::<Projectile>::new();
    let snapshot = SnapshotBuilder::new(map, individuals, projectiles).build(w)?;

    let example = run::Example::builder()
        .world(map_)
        .mod_(PathBuf::from("mods/std1"))
        .snapshot(snapshot);
    #[cfg(feature = "test")]
    let example = example.projectiles(vec![]);
    let _ = example.build().run()?;

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
            )
        })
        .collect()
}
