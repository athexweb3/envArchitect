use axum::{
    extract::{Query, State},
    response::Redirect,
    routing::get,
    Json, Router,
};
use database::Database;
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;

pub fn router() -> Router<Arc<Database>> {
    Router::new()
        .route("/auth/login", get(login_handler))
        .route("/auth/callback", get(callback_handler))
}

async fn login_handler() -> Redirect {
    // TODO: Generate real GitHub OAuth URL with client_id
    Redirect::to("https://github.com/login/oauth/authorize")
}

#[derive(Deserialize)]
struct CallbackQuery {
    code: String,
    state: Option<String>,
}

async fn callback_handler(
    State(_db): State<Arc<Database>>,
    Query(query): Query<CallbackQuery>,
) -> Json<Value> {
    // TODO: Exchange code for token, fetch user info, upsert to DB
    Json(json!({
        "status": "ok",
        "message": "Login logic pending",
        "code": query.code
    }))
}
