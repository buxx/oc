use std::path::PathBuf;

use oc_examples::{
    logging, run,
    snapshot::{EmptyGenerator, SnapshotBuilder, tile::SameTileFiller},
};
use oc_geo::{
    region::RegionXy,
    tile::{TileXy, WorldTileIndex},
};
use oc_individual::{Individual, behavior::Behavior};
use oc_projectile::Projectile;
use oc_root::{GEO_PIXELS_PER_TILE, REGION_HEIGHT, REGION_WIDTH, WORLD_HEIGHT, WORLD_WIDTH};
use oc_world::tile::{Nature, Tile};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Find a way to automatize/standadize that
    if WORLD_WIDTH != 1000 || WORLD_HEIGHT != 1000 || REGION_WIDTH != 100 || REGION_HEIGHT != 100 {
        panic!("Examples must be started from ./examples.sh script");
    }

    logging::setup_logging()?;

    let snapshot = SnapshotBuilder::new(
        SameTileFiller(Nature::ShortGrass),
        individuals,
        EmptyGenerator::<Projectile>::new(),
    )
    .build()?;

    run::Example::builder()
        .world(PathBuf::from("examples/world1"))
        .mod_(PathBuf::from("mods/std1"))
        .snapshot(snapshot)
        .build()
        .run()?;

    Ok(())
}

fn individuals(tiles: &Vec<Tile>) -> Vec<Individual> {
    (0..10)
        .map(|i| {
            let tile_i = WorldTileIndex(i as u64);
            let tile_xy = TileXy::from(tile_i);
            let tile = &tiles[tile_i.0 as usize];

            let position = [
                ((tile_xy.0.0 * GEO_PIXELS_PER_TILE) + GEO_PIXELS_PER_TILE / 2) as f32,
                ((tile_xy.0.1 * GEO_PIXELS_PER_TILE) + GEO_PIXELS_PER_TILE / 2) as f32,
                tile.z as f32,
            ];
            let region: RegionXy = tile_xy.into();
            Individual::new(position, tile_xy, region, Behavior::Idle, vec![])
        })
        .collect()
}
