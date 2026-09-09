use std::{collections::HashMap, path::PathBuf};

use anyhow::Context;
use clap::{Parser, ValueEnum};
use oc_deployment::deployment::{self, place::Placer};
use oc_examples::{logging, run};
use oc_mod::Mod;
use oc_root::{WorldConfig, geo::WorldVec2, physics::Meters};
use oc_world::{load::WorldPath, meta::Meta, place::PlaceName, snapshot::Snapshot};
use uuid::Uuid;

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg()]
    case: Case,

    #[arg(long)]
    phase: Phase,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Case {
    Empty,
    Battle1,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Phase {
    Deployment,
    Battle,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    logging::setup_logging()?;

    let mod_ = Mod::load(&PathBuf::from("mods/std1"), None)?;
    let map_ = PathBuf::from("examples/minidblue");
    let map = oc_world::reader::MapReader::new(&map_);
    let map = map.context(format!("Read map {}", map_.display()))?;
    let map__ = map.build().unwrap();
    let world = Meta::from_file(&map_.meta());
    let world = world.context(format!("Read file {}", map_.meta().display()))?;
    let w = WorldConfig::new(
        map.width().unwrap() as u64,
        map.height().unwrap() as u64,
        Meters(world.geo_meters_per_z),
    );

    let deployments = match args.case {
        Case::Empty => deployment::Deployments::default(),

        Case::Battle1 => deployment::Deployments::from_files(
            &PathBuf::from("examples/deployment1a.yml"),
            &PathBuf::from("examples/deployment1b.yml"),
        )
        .unwrap(),
    };
    let tiles = map.tiles(&w, &mod_).context(format!("Read map tiles"))?;
    let places: HashMap<PlaceName, WorldVec2> = map__
        .places()
        .iter()
        .map(|p| (p.name.clone(), WorldVec2::new(p.x, p.y)))
        .collect();
    let mut snapshot = Snapshot::empty(w.clone()).with_tiles(tiles.clone());

    let placer = match args.phase {
        // FIXME BS NOW: must give spawn zone and attributions to random choose places
        Phase::Deployment => Placer::random(),
        // FIXME BS NOW: must give spawn zone and attributions to random choose places (if unknown place for an squad)
        Phase::Battle => Placer::places(vec![
            // uuids are these from examples/deployment1a.yml & examples/deployment1b.yml
            // places names are from examples/minidblue
            // A
            (
                Uuid::parse_str("f9dbffe2-64ba-4a88-920b-ae883751493e").unwrap(),
                *places.get(&"63".into()).unwrap(),
            ),
            (
                Uuid::parse_str("ca3db430-f7fa-4232-b2a1-2abb85499ae5").unwrap(),
                *places.get(&"50".into()).unwrap(),
            ),
            // B
            (
                Uuid::parse_str("e1b6ec3d-7119-41d1-9f89-a6faf7da5bbb").unwrap(),
                *places.get(&"70".into()).unwrap(),
            ),
            (
                Uuid::parse_str("9dbf54c7-3723-4fe3-a42b-44f3866c09aa").unwrap(),
                *places.get(&"53".into()).unwrap(),
            ),
        ]),
    };

    deployments
        // FIXME BS NOW: si phase "battle", utiliser (inventer) un système de placement (indications
        // sur la carte ? qui pourrait service à l'ia ?)
        // Sinon, créer les mécanique de la phase "deployment"
        .mobilize(&w, &mod_, &mut snapshot, &placer, &tiles)
        .context(format!("Generate snapshot from deployment files"))?;

    let (_, snapshot_path) = tempfile::NamedTempFile::new()?.keep()?;
    snapshot
        .save(&snapshot_path)
        .context(format!("Save snapshot to {}", snapshot_path.display()))?;

    let example = run::Example::builder()
        .world(map_)
        .mod_(PathBuf::from("mods/std1"))
        .snapshot(snapshot_path);
    example.build().run()?;

    Ok(())
}
