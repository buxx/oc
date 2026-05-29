use axum::extract::Path;
use axum::{body::Body, extract::State, http::Response, response::IntoResponse};
use oc_geo::region::WorldRegionIndex;
use oc_root::files;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GetRegionBackgroundPath {
    region: WorldRegionIndex,
}

#[axum::debug_handler]
pub async fn get_region(
    path: Path<GetRegionBackgroundPath>,
    State(state): State<super::State>,
) -> impl IntoResponse {
    let region = path.region;
    let mod_ = state.world().mod_().canonical();
    let world = state.world().meta().canonical();
    let files = files::Files::new(mod_, world).into_server(state.config.cache.clone());
    let path = files.region(region.0);
    let region = tokio::fs::File::open(path).await.unwrap(); // TODO
    let region = tokio_util::io::ReaderStream::new(region);

    Response::builder()
        .header("Content-Type", "application/octet-stream")
        .body(Body::from_stream(region))
        .unwrap()
}

#[axum::debug_handler]
pub async fn get(State(state): State<super::State>) -> impl IntoResponse {
    let mod_ = state.world().mod_().canonical();
    let world = state.world().meta().canonical();
    let files = files::Files::new(mod_, world).into_server(state.config.cache.clone());
    let archive = files.world_archive();
    let archive = tokio::fs::File::open(archive).await.unwrap(); // TODO
    let archive = tokio_util::io::ReaderStream::new(archive);

    Response::builder()
        .header("Content-Type", "application/octet-stream")
        .body(Body::from_stream(archive))
        .unwrap() // TODO
}
