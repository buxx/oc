use std::{fs::File, path::PathBuf};

use bevy::prelude::*;
use fs_extra::dir::CopyOptions;
use oc_geo::region::WorldRegionIndex;
use oc_root::REGIONS_COUNT;

use crate::{
    config::Connect,
    http_to_file, network,
    states::{AppState, Meta, Mod, StaticSource},
    utils::OcPaths,
};

#[derive(Event)]
pub struct Downloaded;

pub struct DownloadingPlugin;

impl Plugin for DownloadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_downloaded)
            .add_systems(Update, download.run_if(in_state(AppState::Downloading)));
    }
}

// TODO: for now, this action is blocking, it should be not and display a progress message
fn download(
    mut commands: Commands,
    mod_: Res<Mod>,
    meta: Res<Meta>,
    static_: Res<StaticSource>,
    network: Res<network::state::State>,
) -> Result<(), BevyError> {
    let Some(static_) = &static_.0 else {
        return Ok(());
    };
    let Some(mod_) = &mod_.0 else { return Ok(()) };
    let Some(meta) = &meta.0 else { return Ok(()) };
    if !network.connected {
        return Ok(());
    };

    let mods = PathBuf::mods();
    let moda = mods.join(mod_.archive());
    let modd = mods.join(mod_.canonical());
    let map_path = PathBuf::maps().join(meta.folder_name());
    let minimap = PathBuf::minimap(&meta);

    match static_ {
        oc_root::static_::StaticSource::Local { mod_, map } => {
            let mod_ = PathBuf::from(mod_);
            let map = PathBuf::from(map);
            let copy = CopyOptions::new().overwrite(true);

            // TODO: copy paste is not very efficient ...
            std::fs::create_dir_all(&modd).unwrap(); // TODO
            fs_extra::copy_items(&[mod_], &modd, &copy).unwrap(); // TODO
            std::fs::create_dir_all(&map_path).unwrap(); // TODO
            let minimapsrc = map.join("minimap.png");
            println!("{minimapsrc:?} -> {minimap:?}");
            std::fs::copy(&minimapsrc, &minimap).unwrap(); // TODO
            let regions: Vec<PathBuf> = (0..REGIONS_COUNT)
                .map(|region| map.join(WorldRegionIndex(region as u64).background_file_name()))
                .collect();
            fs_extra::copy_items(&regions, &map_path, &copy).unwrap(); // TODO
        }
        oc_root::static_::StaticSource::Remote(port) => {
            let Some(Connect::Network(server)) = network.server else {
                return Ok(());
            };

            tracing::info!("Downloading...",);

            let static_ = format!("http://{}:{}", server.ip(), port);

            match modd.exists() {
                true => {
                    tracing::info!(
                        "Mod {} already cached ({})",
                        mod_.canonical(),
                        modd.display()
                    );
                }
                false => {
                    tracing::info!("Download mod {} ({})", mod_.canonical(), moda.display());
                    http_to_file!(format!("{static_}/mod"), &moda);
                    let file = File::open(&moda).unwrap(); // TODO
                    tracing::info!(
                        "Decompress mod {} (into {})",
                        mod_.canonical(),
                        modd.display()
                    );
                    let decoder = flate2::read::GzDecoder::new(file);
                    let mut archive = tar::Archive::new(decoder);
                    archive.unpack(modd).unwrap(); // TODO
                    std::fs::remove_file(moda).unwrap() // TODO
                }
            }

            match minimap.exists() {
                true => {}
                false => {
                    tracing::info!("Download minimap");
                    http_to_file!(format!("{static_}/minimap"), minimap);
                }
            }

            tracing::info!("Download regions");
            for region in 0..REGIONS_COUNT {
                let region = WorldRegionIndex(region as u64);
                let path = map_path.join(region.background_file_name());

                match path.exists() {
                    true => {}
                    false => {
                        http_to_file!(format!("{static_}/region/{}/background", region.0), path);
                    }
                }
            }

            tracing::info!("Download finished");
        }
    }

    commands.trigger(Downloaded);

    Ok(())
}

fn on_downloaded(_: On<Downloaded>, mut app_state: ResMut<NextState<AppState>>) {
    tracing::info!("Download finished");
    tracing::debug!("Entering 'Ingame' state");
    *app_state = NextState::Pending(AppState::InGame);
}

// TODO: make something lean (and which centralize paths for server and client)
// pub struct Synchronizer {
//     source: oc_root::static_::StaticSource,
// }

// impl Synchronizer {
//     pub fn from(source: oc_root::static_::StaticSource) -> Self {
//         Self { source }
//     }

//     pub fn download_mod(&self) -> Result<(), std::io::Error> {
//         let mods = PathBuf::mods();
//         std::fs::create_dir_all(&mods)?;

//         match self.source {
//             oc_root::static_::StaticSource::Local { mod_, map } => {}
//             oc_root::static_::StaticSource::Remote(port) => {}
//         };

//         Ok(())
//     }

//     pub fn download_minimap(&self) -> Result<(), std::io::Error> {
//         Ok(())
//     }

//     pub fn download_regions(&self) -> Result<(), std::io::Error> {
//         Ok(())
//     }
// }
