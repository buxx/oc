use std::{net::SocketAddr, ops::Deref, path::PathBuf, sync::Arc};

use axum::{Router, routing::get};
use derive_more::Constructor;
use message_io::network::Endpoint;

use crate::network::NetworkConfig;

mod minimap;
mod mod_;
mod world;

#[derive(Constructor)]
pub struct Static {
    state: Arc<crate::state::State<Endpoint>>,
    config: NetworkConfig,
}

#[derive(Clone, Constructor)]
pub struct State {
    pub state: Arc<crate::state::State<Endpoint>>,
    pub cache: PathBuf,
}

impl Static {
    pub fn serve(&self, host: SocketAddr) -> Result<(), std::io::Error> {
        let state = State::new(self.state.clone(), self.config.cache.clone());
        let app = Router::new()
            .route("/mod", get(mod_::get))
            .route(
                "/region/{region}/background",
                get(world::get_region_background),
            )
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
