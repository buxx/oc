use std::{
    path::PathBuf,
    sync::{Arc, Mutex, mpsc::channel},
};

use bon::Builder;
use oc_root::static_::StaticSource;
use oc_world::{load::WorldPath, meta::Meta};
use oc_world_server::config::ServerConfig;

use crate::bridge;

#[derive(Debug, Builder)]
pub struct Example {
    world: PathBuf,
    mod_: PathBuf,
}

const SERVER_CACHE: &str = ".cache";

impl Example {
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error + 'static>> {
        let (ready_tx, ready_rx) = channel::<Result<(), String>>();
        let (to_client_tx, to_client_rx) = channel();
        let (to_server_tx, to_server_rx) = channel();
        let world = Meta::from_file(&self.world.meta())?;

        tracing::info!("Start server");

        let cache = PathBuf::from(SERVER_CACHE);
        let static_ = StaticSource::Local {
            mod_: self.mod_.display().to_string(),
            map: cache
                .join("maps")
                .join(world.folder_name())
                .display()
                .to_string(),
        };
        let config = ServerConfig::builder()
            .world(self.world.clone())
            .mod_(self.mod_.clone())
            .cache(cache.clone())
            .print_ticks(false)
            .static_(static_)
            .build();
        let state = oc_world_server::state::init::<()>(config.clone())?;
        let state = Arc::new(state);

        std::thread::spawn(move || {
            oc_world_server::run::run(config, state, to_server_rx, to_client_tx, ready_tx)
        });

        tracing::info!("Waiting server ...");
        match ready_rx.recv()? {
            Ok(_) => {}
            Err(error) => return Err(format!("Failed to start server: {error}").into()),
        };

        let (client_tx2, server_rx2) = bridge::bridge(to_client_rx, to_server_tx);

        tracing::info!("Start gui");
        let server_rx2 = Arc::new(Mutex::new(server_rx2));
        let connect = oc_battle_gui::config::Connect::Embedded(client_tx2, server_rx2);
        let config = oc_battle_gui::config::Config_::builder()
            .autoconnect(connect)
            .build();

        oc_battle_gui::run::run(config)?;
        Ok(())
    }
}
