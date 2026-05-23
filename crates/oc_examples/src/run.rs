use std::{
    path::PathBuf,
    sync::{Arc, Mutex, mpsc::channel},
};

use anyhow::Context;
use bon::Builder;
use oc_root::{files::Files, static_::StaticSource};
use oc_world::{load::WorldPath, meta::Meta};
use oc_world_server::config::ServerConfig;
#[cfg(feature = "test")]
use oc_world_server::tracker::Tracker;

use crate::bridge;

#[cfg(feature = "test")]
type Result_ = Result<Tracker, anyhow::Error>;

#[cfg(not(feature = "test"))]
type Result_ = Result<(), anyhow::Error>;

#[derive(Builder)]
pub struct Example {
    mod_: PathBuf,
    world: PathBuf,
    snapshot: PathBuf,
    install: Option<Box<dyn Fn(&mut bevy::app::App)>>,
}

impl Example {
    pub fn run(self) -> Result_ {
        let cache = PathBuf::from("assets/cache_");
        let files = Files::new("".to_string(), "".to_string()).into_server(cache.clone());
        std::fs::create_dir_all(files.mods())
            .context(format!("Create dir {}", files.mods().display()))?;
        std::fs::create_dir_all(files.worlds())
            .context(format!("Create dir {}", files.worlds().display()))?;

        #[cfg(feature = "test")]
        let tracker = Tracker::default();

        let (ready_tx, ready_rx) = channel::<std::result::Result<(), String>>();
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
            .cache(cache)
            .print_ticks(false)
            .static_(static_)
            .snapshot(self.snapshot.clone())
            .build();
        let state = oc_world_server::state::init::<()>(config.clone())?;
        let state = Arc::new(state);

        {
            #[cfg(feature = "test")]
            let tracker = tracker.clone();
            std::thread::spawn(move || {
                oc_world_server::runner::Runner::new(
                    config,
                    state,
                    to_client_tx,
                    #[cfg(feature = "test")]
                    tracker,
                )
                .run(to_server_rx, ready_tx);
            });
        }

        tracing::info!("Waiting server ...");
        let _ = ready_rx.recv().context("Wait server ready")?;

        let (client_tx2, server_rx2) = bridge::bridge(to_client_rx, to_server_tx);

        tracing::info!("Start gui");
        let server_rx2 = Arc::new(Mutex::new(server_rx2));
        let connect = oc_battle_gui::config::Connect::Embedded(client_tx2, server_rx2);
        let config = oc_battle_gui::config::Config_::builder().autoconnect(connect);
        let config = config.build();

        oc_battle_gui::run::run()
            .config(config)
            .maybe_install(self.install)
            .call();

        #[cfg(feature = "test")]
        {
            Ok(tracker)
        }

        #[cfg(not(feature = "test"))]
        {
            Ok(())
        }
    }
}
