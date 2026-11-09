use axum::{extract::State, response::IntoResponse};
use oc_root::files::File;

#[axum::debug_handler]
pub async fn get(State(state): State<super::State>) -> impl IntoResponse {
    super::stream(state.files().fs(File::Minimap)).await
}

#[axum::debug_handler]
pub async fn get_hud(State(state): State<super::State>) -> impl IntoResponse {
    super::stream(state.files().fs(File::HudMinimap)).await
}
