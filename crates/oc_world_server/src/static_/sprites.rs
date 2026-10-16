use axum::{body::Body, extract::State, http::Response, response::IntoResponse};
use oc_root::files;

#[axum::debug_handler]
pub async fn get_soldiers_outline(State(state): State<super::State>) -> impl IntoResponse {
    let mod_ = state.world().mod_().canonical();
    let files = files::Files::new(mod_, "".to_string()).into_server(state.config.cache.clone());
    let outline = tokio::fs::File::open(files.soldier_outline())
        .await
        .unwrap(); // TODO
    let outline = tokio_util::io::ReaderStream::new(outline);

    Response::builder()
        .header("Content-Type", "application/octet-stream")
        .body(Body::from_stream(outline))
        .unwrap() // TODO
}
