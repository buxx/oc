use bevy::prelude::*;
use oc_root::{
    REGIONS_COUNT,
    files::{self},
};

use crate::{
    http_to_file, network,
    states::{AppState, Meta, Mod, StaticSource},
    utils::untar,
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
// TODO: refact this ugly non refactored download / cache / assets code
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
    let Some(connect) = network.server.clone() else {
        return Ok(());
    };

    let mod_ = mod_.canonical();
    let world = meta.canonical();
    let files = files::Files::new(mod_, world).into_gui(static_.clone(), connect.into());

    tracing::info!("Download");

    ensure_file(&files, files::File::Mod).unwrap(); // TODO
    ensure_file(&files, files::File::World).unwrap(); // TODO
    ensure_file(&files, files::File::Minimap).unwrap(); // TODO
    for region in 0..REGIONS_COUNT {
        ensure_file(&files, files::File::Region(region as u64)).unwrap(); // TODO
    }

    // let mods = PathBuf::mods();
    // let moda = mods.join(mod_.archive());
    // let modd = mods.join(mod_.canonical());
    // let worlds = PathBuf::worlds();
    // let worlda = worlds.join(meta.archive());
    // let worldd = worlds.clone();
    // let map_path = PathBuf::maps().join(meta.canonical());
    // let minimap = PathBuf::minimap(&meta);

    // match static_ {
    //     oc_root::static_::StaticSource::Local { mod_, map, world } => {
    //         let mod_ = PathBuf::from(mod_);
    //         let world = PathBuf::from(world);
    //         let map = PathBuf::from(map);
    //         let copy = CopyOptions::new().overwrite(true);

    //         // TODO: copy paste is not very efficient ...
    //         std::fs::create_dir_all(&modd).unwrap(); // TODO
    //         fs_extra::copy_items(&[mod_], &modd, &copy).unwrap(); // TODO
    //         std::fs::create_dir_all(&worldd).unwrap(); // TODO
    //         println!("{} -> {}", world.display(), worldd.display());
    //         fs_extra::copy_items(&[world], &worldd, &copy).unwrap(); // TODO
    //         std::fs::create_dir_all(&map_path).unwrap(); // TODO
    //         let minimapsrc = map.join("minimap.png");
    //         std::fs::copy(&minimapsrc, &minimap).unwrap(); // TODO
    //         let regions: Vec<PathBuf> = (0..REGIONS_COUNT)
    //             .map(|region| map.join(WorldRegionIndex(region as u64).background_file_name()))
    //             .collect();
    //         fs_extra::copy_items(&regions, &map_path, &copy).unwrap(); // TODO
    //     }
    //     oc_root::static_::StaticSource::Remote(port) => {
    //         let Some(Connect::Network(server)) = network.server else {
    //             return Ok(());
    //         };

    //         tracing::info!("Downloading...",);

    //         let static_ = format!("http://{}:{}", server.ip(), port);

    //         match modd.exists() {
    //             true => {
    //                 tracing::info!(
    //                     "Mod {} already cached ({})",
    //                     mod_.canonical(),
    //                     modd.display()
    //                 );
    //             }
    //             false => {
    //                 tracing::info!("Download mod {} ({})", mod_.canonical(), moda.display());
    //                 http_to_file!(format!("{static_}/mod"), &moda);
    //                 let file = File::open(&moda).unwrap(); // TODO
    //                 tracing::info!(
    //                     "Decompress mod {} (into {})",
    //                     mod_.canonical(),
    //                     modd.display()
    //                 );
    //                 let decoder = flate2::read::GzDecoder::new(file);
    //                 let mut archive = tar::Archive::new(decoder);
    //                 archive.unpack(modd).unwrap(); // TODO
    //                 std::fs::remove_file(moda).unwrap() // TODO
    //             }
    //         }

    //         match worldd.exists() {
    //             true => {
    //                 tracing::info!(
    //                     "World {} already cached ({})",
    //                     meta.canonical(),
    //                     worldd.display()
    //                 );
    //             }
    //             false => {
    //                 tracing::info!("Download world {} ({})", meta.canonical(), worlda.display());
    //                 http_to_file!(format!("{static_}/world"), &worlda);
    //                 let file = File::open(&worlda).unwrap(); // TODO
    //                 tracing::info!(
    //                     "Decompress world {} (into {})",
    //                     meta.canonical(),
    //                     worldd.display()
    //                 );
    //                 let decoder = flate2::read::GzDecoder::new(file);
    //                 let mut archive = tar::Archive::new(decoder);
    //                 archive.unpack(worldd).unwrap(); // TODO
    //                 std::fs::remove_file(worlda).unwrap() // TODO
    //             }
    //         }

    //         match minimap.exists() {
    //             true => {}
    //             false => {
    //                 tracing::info!("Download minimap");
    //                 http_to_file!(format!("{static_}/minimap"), minimap);
    //             }
    //         }

    //         tracing::info!("Download regions");
    //         for region in 0..REGIONS_COUNT {
    //             let region = WorldRegionIndex(region as u64);
    //             let path = map_path.join(region.background_file_name());

    //             match path.exists() {
    //                 true => {}
    //                 false => {
    //                     http_to_file!(format!("{static_}/region/{}/background", region.0), path);
    //                 }
    //             }
    //         }

    //         tracing::info!("Download finished");
    //     }
    // }

    commands.trigger(Downloaded);

    Ok(())
}

fn ensure_file(
    files: &files::FilesAsGui,
    file: files::File,
) -> Result<(), Box<dyn std::error::Error>> {
    let Some((sync, target)) = files.method(file.clone()) else {
        tracing::info!("Use local {file}");
        return Ok(());
    };

    tracing::info!("Download {file}");
    tracing::debug!("Ensure file with sync {sync} ({})", target.display());

    match target.exists() {
        false => {
            tracing::debug!("File {} doesn't exist", target.display());
            std::fs::create_dir_all(target.parent().unwrap()).unwrap(); // TODO

            match sync {
                files::Sync::DirectDownload(url) => {
                    tracing::debug!("Direct download from {url}");
                    http_to_file!(url, &target);
                }
                files::Sync::ArchiveDownload(url) => {
                    tracing::debug!("Download archive from {url}");
                    let (_, path) = tempfile::NamedTempFile::new().unwrap().keep().unwrap(); // TODO
                    http_to_file!(url, &path);
                    untar(&path, &target)?;
                }
            }
        }
        true => {}
    };

    Ok(())
}

fn on_downloaded(_: On<Downloaded>, mut app_state: ResMut<NextState<AppState>>) {
    tracing::info!("Download finished");
    tracing::debug!("Entering 'Ingame' state");
    *app_state = NextState::Pending(AppState::InGame);
}
