use crate::services::auth_service::AuthService;
use crate::state::AppState;
use axum::http::StatusCode;
use axum::{
    extract::{Query, State},
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use serde_json::json;
use tower_sessions::Session;
use uuid::Uuid;

pub async fn initiate_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Response {
    let auth_service = AuthService::new(state.db.clone());

    let host = headers
        .get("host")
        .and_then(|h| h.to_str().ok())
        .map(|h| {
            if h.contains("localhost:3000") {
                "http://localhost:3001".to_string()
            } else {
                format!("http://{}", h)
            }
        })
        .unwrap_or_else(|| "http://localhost:3001".to_string());

    match auth_service.initiate_device_flow(&host).await {
        Ok(response) => Json::<shared::dto::AuthDeviceResponse>(response).into_response(),
        Err(e) => {
            tracing::error!("Failed to initiate device flow: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "status": "error",
                    "message": e.to_string()
                })),
            )
                .into_response()
        }
    }
}

#[derive(Deserialize)]
pub struct VerifyQuery {
    pub user_code: String,
}

pub async fn verify_handler(
    State(state): State<AppState>,
    session: Session,
    Query(query): Query<VerifyQuery>,
) -> Response {
    let auth_service = AuthService::new(state.db.clone());

    // Check if user is logged into the portal (cookie session)
    let user_id: Option<Uuid> = session.get("user_id").await.unwrap_or(None);

    if let Some(uid) = user_id {
        // Link device code to user
        match auth_service.authorize_device(&query.user_code, uid).await {
            Ok(_) => Json(json!({
                "status": "success",
                "message": "Device authorized successfully. You can return to your CLI."
            }))
            .into_response(),
            Err(e) => (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "status": "error",
                    "message": e.to_string()
                })),
            )
                .into_response(),
        }
    } else {
        Json(json!({
            "status": "pending_login",
            "message": "Please log in to authorize this device.",
            "user_code": query.user_code
        }))
        .into_response()
    }
}
