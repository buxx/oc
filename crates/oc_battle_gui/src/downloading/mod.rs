use bevy::prelude::*;
use oc_root::files::{self};
use oc_utils::let_some;

use crate::{
    http_to_file, network,
    states::{AppState, GameConfig, PointerIn},
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
    g: Res<GameConfig>,
    network: Res<network::state::State>,
    mut world_: ResMut<World>,
) -> Result<(), BevyError> {
    let_some!(g = &g.0, return Ok(()));
    let_some!(connect = network.server.clone(), return Ok(()));

    let mod__ = g.mod_.canonical();
    let world = g.meta.canonical();
    let files = files::Files::new(mod__, world).into_gui(g.static_.clone(), connect.into());

    tracing::info!("Download");
    let region_width = g.w.region_width;
    let region_height = g.w.region_height;

    ensure_file(&files, files::File::Mod, region_width, region_height).unwrap(); // TODO
    ensure_file(&files, files::File::World, region_width, region_height).unwrap(); // TODO
    ensure_file(&files, files::File::Minimap, region_width, region_height).unwrap(); // TODO
    for region in 0..g.w.regions_count {
        ensure_file(
            &files,
            files::File::Region(region),
            region_width,
            region_height,
        )
        .unwrap(); // TODO
    }

    // FIXME: check tile size
    let terrain =
        oc_world::terrain::Terrain::load(&files.terrain_tsx(), g.w.clone(), &g.mod_).unwrap(); // TODO
    tracing::trace!(name="downloading-terrain", terrain=?terrain);
    world_.terrain = Some(terrain); // TODO

    commands.trigger(Downloaded);

    Ok(())
}

fn ensure_file(
    files: &files::FilesAsGui,
    file: files::File,
    region_width: u64,
    region_height: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let Some((sync, target)) = files.method(file, region_width, region_height) else {
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

fn on_downloaded(
    _: On<Downloaded>,
    mut app_state: ResMut<NextState<AppState>>,
    mut pointer: ResMut<NextState<PointerIn>>,
) {
    tracing::info!("Download finished");
    tracing::debug!("Entering 'Ingame' state");
    *app_state = NextState::Pending(AppState::InGame);
    *pointer = NextState::Pending(PointerIn::Battle)
}
