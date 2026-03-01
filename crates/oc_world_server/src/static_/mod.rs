use std::{net::SocketAddr, ops::Deref, path::PathBuf, sync::Arc};

use axum::{Router, routing::get};
use derive_more::Constructor;

mod minimap;
mod world;

#[derive(Constructor)]
pub struct Static {
    state: Arc<crate::state::State>,
    cache: PathBuf,
}

#[derive(Clone, Constructor)]
pub struct State {
    pub state: Arc<crate::state::State>,
    pub cache: PathBuf,
}

impl Static {
    pub fn serve(&self, host: SocketAddr) -> Result<(), std::io::Error> {
        let state = State::new(self.state.clone(), self.cache.clone());
        let app = Router::new()
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
    type Target = crate::state::State;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}
