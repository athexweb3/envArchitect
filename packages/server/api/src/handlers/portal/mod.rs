use axum::{extract::State, Json};
use axum::{routing::get, Router};
use serde_json::{json, Value};

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/portal/stats", get(stats_handler))
}

pub async fn stats_handler(State(_state): State<AppState>) -> Json<Value> {
    Json(json!({
        "status": "ok",
        "message": "Dashboard stats placeholder"
    }))
}
