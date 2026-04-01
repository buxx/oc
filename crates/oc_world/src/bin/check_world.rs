use std::path::PathBuf;

use clap::Parser;
use oc_world::reader;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Folder which contain world
    #[clap()]
    pub path: PathBuf,
}

fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();
    setup_logging()?;

    match check(args) {
        Err(error) => {
            tracing::error!("{}", error);
        }
        _ => tracing::info!("No error found"),
    }

    Ok(())
}

fn check(args: Args) -> Result<(), anyhow::Error> {
    tracing::info!("Check map structure");
    let map = reader::MapReader::new(&args.path)?;

    tracing::info!("Check map tiles");
    map.tiles()?;

    Ok(())
}

fn setup_logging() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env()?,
        )
        .init();
    Ok(())
}
