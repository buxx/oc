use bevy::prelude::*;
use oc_root::{
    REGIONS_COUNT,
    files::{self},
};

use crate::{
    http_to_file, network,
    states::{AppState, Meta, Mod, StaticSource},
    utils::untar,
    world::World,
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
    mut world_: ResMut<World>,
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

    // FIXME: check tile size
    let terrain = oc_world::terrain::Terrain::load(&files.terrain_tsx()).unwrap(); // TODO
    tracing::trace!(name="downloading-terrain", terrain=?terrain);
    world_.terrain = Some(terrain); // TODO

    commands.trigger(Downloaded);

    Ok(())
}

fn ensure_file(
    files: &files::FilesAsGui,
    file: files::File,
) -> Result<(), Box<dyn std::error::Error>> {
    let Some((sync, target)) = files.method(file) else {
        tracing::info!("Use local {file}");
        return Ok(());
    };

    tracing::info!("Download {file}");
    tracing::debug!("Ensure file with sync {sync} ({})", target.display());

    if !target.exists() {
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
    };

    Ok(())
}

fn on_downloaded(_: On<Downloaded>, mut app_state: ResMut<NextState<AppState>>) {
    tracing::info!("Download finished");
    tracing::debug!("Entering 'Ingame' state");
    *app_state = NextState::Pending(AppState::InGame);
}
