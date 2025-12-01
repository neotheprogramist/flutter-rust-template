use axum::{Json, extract::State, response::IntoResponse};
use serde::{Deserialize, Serialize};

use crate::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

pub async fn healthz(State(state): State<AppState>) -> impl IntoResponse {
    Json(HealthResponse {
        status: "healthy".into(),
        version: state.version.into(),
    })
}
