use std::path::PathBuf;

use bevy::prelude::*;
use oc_geo::region::WorldRegionIndex;
use oc_root::REGIONS_COUNT;

use crate::{
    Args, http_to_file, network,
    states::{AppState, Config, Meta},
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
    meta: Res<Meta>,
    config: Res<Config>,
    network: Res<network::state::State>,
) -> Result<(), BevyError> {
    let Some(config) = &config.0 else {
        return Ok(());
    };
    let Some(meta) = &meta.0 else { return Ok(()) };
    let Some(network) = &network.connected else {
        return Ok(());
    };

    tracing::info!(
        "Start downloading ({} region backgrounds) ...",
        REGIONS_COUNT
    );

    let static_ = format!("http://{}:{}", network.ip(), config.static_);
    let mut counter = 0;
    let path = PathBuf::maps();
    let minimap = PathBuf::minimap(&meta);

    std::fs::create_dir_all(&path)?;

    match minimap.exists() {
        true => {}
        false => {
            http_to_file!(format!("{static_}/minimap"), minimap);
        }
    }

    for region in 0..REGIONS_COUNT {
        let region = WorldRegionIndex(region as u64);
        let path = path.join(region.background_file_name());

        match path.exists() {
            true => {}
            false => {
                http_to_file!(format!("{static_}/region/{}/background", region.0), path);
                counter += 1;
            }
        }
    }

    tracing::info!(
        "Download finished ({} downloaded, {} in cache)",
        counter,
        REGIONS_COUNT - counter
    );
    commands.trigger(Downloaded);

    Ok(())
}

fn on_downloaded(_: On<Downloaded>, mut app_state: ResMut<NextState<AppState>>) {
    tracing::info!("Download finished");
    tracing::debug!("Entering 'Ingame' state");
    *app_state = NextState::Pending(AppState::InGame);
}
