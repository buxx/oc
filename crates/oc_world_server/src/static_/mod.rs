use std::{net::SocketAddr, ops::Deref, sync::Arc};

use axum::{Router, routing::get};
use derive_more::Constructor;
use message_io::network::Endpoint;

use crate::{config::ServerConfig, network::NetworkConfig};

mod minimap;
mod mod_;
mod world;

#[derive(Constructor)]
pub struct Static {
    state: Arc<crate::state::State<Endpoint>>,
    network: NetworkConfig,
    config: ServerConfig,
}

#[derive(Clone, Constructor)]
pub struct State {
    pub state: Arc<crate::state::State<Endpoint>>,
    pub _network: NetworkConfig,
    pub config: ServerConfig,
}

impl Static {
    pub fn serve(&self, host: SocketAddr) -> Result<(), std::io::Error> {
        let state = State::new(
            self.state.clone(),
            self.network.clone(),
            self.config.clone(),
        );
        let app = Router::new()
            .route("/mod", get(mod_::get))
            .route("/world", get(world::get))
            .route("/region/{region}", get(world::get_region))
            .route("/minimap", get(minimap::get))
            .with_state(state);

        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async {
            let listener = tokio::net::TcpListener::bind(host).await.unwrap();
            tracing::info!("Serve statics on http://{host}");
            axum::serve(listener, app).await.unwrap();
        });

        Ok(())
    }
}

impl Deref for State {
    type Target = crate::state::State<Endpoint>;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}
