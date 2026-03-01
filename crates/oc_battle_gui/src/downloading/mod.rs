use std::fs::create_dir_all;

use bevy::prelude::*;
use oc_geo::region::WorldRegionIndex;
use oc_root::REGIONS_COUNT;

use crate::{Args, states::AppState, states::Meta};

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
fn download(mut commands: Commands, args: Res<Args>, meta: Res<Meta>) {
    let Some(meta) = &meta.0 else { return };
    tracing::info!(
        "Start downloading ({} region backgrounds) ...",
        REGIONS_COUNT
    );
    let mut counter = 0;

    for region in 0..REGIONS_COUNT {
        let region = WorldRegionIndex(region as u64);
        let path = args.0.cache.join(meta.folder_name());
        let path = path.join(region.background_file_name());
        std::fs::create_dir_all(&path).unwrap();

        match path.exists() {
            true => {}
            false => {
                // TODO: host given by server through network ?
                // TODO: unwraps
                let url = format!("http://127.0.0.1:6590/region/{}/background", region.0);
                let mut resp = reqwest::blocking::get(url).unwrap();
                let mut file = std::fs::File::create(path).unwrap();
                std::io::copy(&mut resp, &mut file).unwrap();
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
}

fn on_downloaded(_: On<Downloaded>, mut app_state: ResMut<NextState<AppState>>) {
    tracing::info!("Download finished");
    tracing::debug!("Entering 'Ingame' state");
    *app_state = NextState::Pending(AppState::InGame);
}
