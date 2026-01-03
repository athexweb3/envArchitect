use crate::middleware::auth::AuthUser;
use crate::services::auth_service::AuthService;
use crate::state::AppState;
use axum::http::StatusCode;
use axum::{
    extract::{Query, State},
    response::{IntoResponse, Response},
    Extension, Json,
};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
pub struct PollQuery {
    pub device_code: String,
}

pub async fn poll_handler(
    State(state): State<AppState>,
    Query(query): Query<PollQuery>,
) -> Response {
    let auth_service = AuthService::new(state.db.clone());

    match auth_service.poll_device_flow(&query.device_code).await {
        Ok(Some(response)) => Json(serde_json::to_value(response).unwrap()).into_response(),
        Ok(None) => Json(json!({
            "status": "pending",
            "message": "Authorization pending"
        }))
        .into_response(),
        Err(e) => {
            tracing::error!("Auth error: {:?}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "status": "error",
                    "message": e.to_string()
                })),
            )
                .into_response()
        }
    }
}
pub async fn me_handler(State(state): State<AppState>, headers: axum::http::HeaderMap) -> Response {
    let auth_header = headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "));

    let token = match auth_header {
        Some(t) => t,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                "Missing or invalid Authorization header",
            )
                .into_response()
        }
    };

    let auth_service = AuthService::new(state.db.clone());

    let claims = match auth_service.verify_token(token) {
        Ok(c) => c,
        Err(_) => return (StatusCode::UNAUTHORIZED, "Invalid or expired token").into_response(),
    };

    let user_id = match uuid::Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => return (StatusCode::UNAUTHORIZED, "Invalid user id in token").into_response(),
    };

    match auth_service.find_user_by_id(user_id).await {
        Ok(Some(user)) => Json(user).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "User not found").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn ghcr_token_handler(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> Response {
    let auth_service = AuthService::new(state.db.clone());

    match auth_service.get_ghcr_token(auth_user.0.id).await {
        Ok(Some(token)) => Json(json!({ "token": token })).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            "GitHub token not found. Please log in again.",
        )
            .into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

#[derive(Deserialize)]
pub struct StorePatRequest {
    pub pat: String,
}

pub async fn store_ghcr_pat_handler(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(payload): Json<StorePatRequest>,
) -> Response {
    let auth_service = AuthService::new(state.db.clone());

    match auth_service
        .store_ghcr_pat(auth_user.0.id, &payload.pat)
        .await
    {
        Ok(_) => Json(json!({ "message": "GHCR PAT stored securely." })).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}
