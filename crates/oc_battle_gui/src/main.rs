use std::{net::SocketAddr, path::PathBuf};

use clap::{Parser, Subcommand};
use oc_battle_gui::{
    config::{Config_, Connect},
    embedded::Embedded,
    run,
};
use oc_root::{files::Roots, side::Side};
use oc_world_server::args::ServerArgs;

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub mode: Option<Mode>,
    /// Side to play when connect auto
    #[clap(long, value_enum, default_value = "a", global = true)]
    pub side: Side,
    #[cfg(feature = "debug")]
    #[clap(long, global = true)]
    pub debug_auto_end: bool,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Mode {
    /// Connect to a world server through network
    Network {
        /// World server address (like 127.0.0.1:6589)
        address: SocketAddr,
        /// Directory where files downloaded from server are stored (default: user data directory)
        #[clap(long)]
        data_dir: Option<PathBuf>,
    },
    /// Run a world server in this process
    Embedded(Box<ServerArgs>),
}

impl Args {
    fn config(self) -> anyhow::Result<Config_> {
        let (roots, connect) = match self.mode {
            None => (Roots::under(&Roots::default_data_dir())?, None),
            Some(Mode::Network { address, data_dir }) => {
                let data_dir = data_dir.unwrap_or_else(Roots::default_data_dir);
                (Roots::under(&data_dir)?, Some(Connect::Network(address)))
            }
            Some(Mode::Embedded(args)) => {
                let (roots, connect) = Embedded::builder()
                    .args(*args)
                    .side(self.side)
                    .build()
                    .start()?;
                (roots, Some(connect))
            }
        };

        Ok(Config_::builder()
            .maybe_auto_connect(connect.map(|c| (c, self.side)))
            .roots(roots)
            .build())
    }
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    #[cfg(feature = "debug")]
    let install = args.debug_auto_end.then(|| -> run::InstallFn {
        Box::new(|app| {
            app.add_plugins(oc_battle_gui::auto_end::AutoEndPlugin);
        })
    });
    #[cfg(not(feature = "debug"))]
    let install = None;

    let config = args.config()?;
    run::run()
        .config(config)
        .maybe_install(install)
        .logging(std::env::var("RUST_LOG").is_ok())
        .call();

    Ok(())
}
