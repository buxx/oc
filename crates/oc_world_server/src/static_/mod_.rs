use axum::{body::Body, extract::State, http::Response, response::IntoResponse};
use oc_mod::MOD_DIR;

#[axum::debug_handler]
pub async fn get(State(state): State<super::State>) -> impl IntoResponse {
    let name = state.world().mod_().archive();
    let path = state.cache.join(MOD_DIR).join(name);
    let archive = tokio::fs::File::open(path).await.unwrap(); // TODO
    let archive = tokio_util::io::ReaderStream::new(archive);

    Response::builder()
        .header("Content-Type", "application/octet-stream")
        .body(Body::from_stream(archive))
        .unwrap() // TODO
}
