use std::{
    path::PathBuf,
    sync::{Arc, Mutex, mpsc::channel},
};

use anyhow::Context;
use bon::Builder;
use oc_root::{files::Files, static_::StaticSource};
use oc_world::{load::WorldPath, meta::Meta};
use oc_world_server::config::ServerConfig;

use crate::bridge;

#[derive(Debug, Builder)]
pub struct Example {
    mod_: PathBuf,
    world: PathBuf,
    snapshot: PathBuf,
}

impl Example {
    pub fn run(&self) -> Result<(), anyhow::Error> {
        let files = Files::new("".to_string(), "".to_string()).into_server(PathBuf::from(".cache"));
        std::fs::create_dir_all(files.mods())
            .context(format!("Create dir {}", files.mods().display()))?;
        std::fs::create_dir_all(files.worlds())
            .context(format!("Create dir {}", files.worlds().display()))?;

        let (ready_tx, ready_rx) = channel::<Result<(), String>>();
        let (to_client_tx, to_client_rx) = channel();
        let (to_server_tx, to_server_rx) = channel();
        let world = Meta::from_file(&self.world.meta())
            .context(format!("Read file {}", self.world.meta().display()))?;

        tracing::info!("Start server");

        // FIXME BS NOW: use Files
        let static_ = StaticSource::Local {
            mod_: self.mod_.display().to_string(),
            world: world.name.clone(),
        };
        let config = ServerConfig::builder()
            .world(self.world.clone())
            .mod_(self.mod_.clone())
            .cache(PathBuf::from(".cache"))
            .print_ticks(false)
            .static_(static_)
            .snapshot(self.snapshot.clone())
            .build();
        let state = oc_world_server::state::init::<()>(config.clone())?;
        let state = Arc::new(state);

        std::thread::spawn(move || {
            oc_world_server::run::run(config, state, to_server_rx, to_client_tx, ready_tx)
        });

        tracing::info!("Waiting server ...");
        let _ = ready_rx.recv().context("Wait server ready")?;

        let (client_tx2, server_rx2) = bridge::bridge(to_client_rx, to_server_tx);

        tracing::info!("Start gui");
        let server_rx2 = Arc::new(Mutex::new(server_rx2));
        let connect = oc_battle_gui::config::Connect::Embedded(client_tx2, server_rx2);
        let config = oc_battle_gui::config::Config_::builder()
            .autoconnect(connect)
            .build();

        oc_battle_gui::run::run(config);
        Ok(())
    }
}
