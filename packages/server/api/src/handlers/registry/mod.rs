use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde_json::json;

pub mod dependents;
pub mod publish;
pub mod search;

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/v1/publish", post(publish::publish_plugin))
        .route("/v1/search", get(search::search_handler))
        .route(
            "/v1/plugins/:name/dependents",
            get(dependents::list_dependents),
        )
}

pub enum ServiceError {
    DatabaseError(String),
    #[allow(dead_code)]
    BadRequest(String),
    Forbidden(String),
    InternalError(String),
}

impl IntoResponse for ServiceError {
    fn into_response(self) -> Response {
        let (status, msg) = match self {
            ServiceError::DatabaseError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e),
            ServiceError::BadRequest(e) => (StatusCode::BAD_REQUEST, e),
            ServiceError::Forbidden(e) => (StatusCode::FORBIDDEN, e),
            ServiceError::InternalError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e),
        };

        (status, Json(json!({ "error": msg }))).into_response()
    }
}
