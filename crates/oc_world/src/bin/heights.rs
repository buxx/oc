use std::path::PathBuf;

use clap::Parser;
use oc_root::WorldConfig;
use oc_world::reader::MapReader;
use serde::Serialize;

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
struct Args {
    #[clap()]
    world: PathBuf,

    /// JSON output file path
    #[clap(default_value = "./heights.json")]
    output: PathBuf,
}

#[derive(Serialize)]
struct Heights {
    width: u64,
    height: u64,
    tile_size: u64,
    data: Vec<u8>,
}

fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();
    let reader = MapReader::new(&args.world).unwrap();
    let (width, height) = (
        reader.width().unwrap() as u64,
        reader.height().unwrap() as u64,
    );
    let w = WorldConfig::new(width, height);
    let tiles = reader.tiles(&w).unwrap();
    let data: Vec<u8> = tiles.iter().map(|t| t.z).collect();
    let tile_size = 5; // FIXME

    let data = Heights {
        width,
        height,
        tile_size,
        data,
    };
    let raw = serde_json::to_string(&data).unwrap();
    std::fs::write(args.output, &raw).unwrap();

    Ok(())
}
