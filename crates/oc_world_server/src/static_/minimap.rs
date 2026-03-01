use axum::{body::Body, extract::State, http::Response, response::IntoResponse};

#[axum::debug_handler]
pub async fn get(State(state): State<super::State>) -> impl IntoResponse {
    let path = state.world().meta().folder_name();
    let path = state.cache.join(path).join("minimap.png"); // TODO: place this name a unique place
    let minimap = tokio::fs::File::open(path).await.unwrap();
    let minimap = tokio_util::io::ReaderStream::new(minimap);

    Response::builder()
        .header("Content-Type", "application/octet-stream")
        .body(Body::from_stream(minimap))
        .unwrap()
}
