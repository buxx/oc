use axum::{
    body::Body,
    extract::{Path, State},
    http::Response,
    response::IntoResponse,
};
use oc_geo::region::WorldRegionIndex;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GetRegionBackgroundPath {
    region: WorldRegionIndex,
}

#[axum::debug_handler]
pub async fn get_region_background(
    path: Path<GetRegionBackgroundPath>,
    State(state): State<super::State>,
) -> impl IntoResponse {
    let region = path.region.background_file_name();
    let path = state.world().meta().folder_name();
    let path = state.cache.join(path).join(region);
    let region = tokio::fs::File::open(path).await.unwrap();
    let region = tokio_util::io::ReaderStream::new(region);

    Response::builder()
        .header("Content-Type", "application/octet-stream")
        .body(Body::from_stream(region))
        .unwrap()
}
