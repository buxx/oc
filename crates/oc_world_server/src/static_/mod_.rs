use axum::{body::Body, extract::State, http::Response, response::IntoResponse};
use oc_root::files;

#[axum::debug_handler]
pub async fn get(State(state): State<super::State>) -> impl IntoResponse {
    let mod_ = state.world().mod_().canonical();
    let world = state.world().meta().canonical();
    let files = files::Files::new(mod_, world).into_server(state.config.cache.clone());
    let archive = files.mod_archive();
    let archive = tokio::fs::File::open(archive).await.unwrap(); // TODO
    let archive = tokio_util::io::ReaderStream::new(archive);

    Response::builder()
        .header("Content-Type", "application/octet-stream")
        .body(Body::from_stream(archive))
        .unwrap() // TODO
}
