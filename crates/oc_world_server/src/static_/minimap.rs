use axum::{body::Body, extract::State, http::Response, response::IntoResponse};
use oc_root::files;

#[axum::debug_handler]
pub async fn get(State(state): State<super::State>) -> impl IntoResponse {
    let mod_ = state.world().mod_().canonical();
    let world = state.world().meta().canonical();
    let files = files::Files::new(mod_, world).into_server(state.config.cache.clone());
    let minimap = files.minimap();
    let minimap = tokio::fs::File::open(minimap).await.unwrap(); // TODO
    let minimap = tokio_util::io::ReaderStream::new(minimap);

    Response::builder()
        .header("Content-Type", "application/octet-stream")
        .body(Body::from_stream(minimap))
        .unwrap() // TODO
}
