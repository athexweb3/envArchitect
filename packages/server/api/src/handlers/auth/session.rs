use crate::services::auth_service::AuthService;
use crate::state::AppState;
use axum::{extract::State, http::StatusCode, response::Json};
use serde_json::{json, Value};
use tower_sessions::Session;
use uuid::Uuid;

pub async fn session_handler(
    State(state): State<AppState>,
    session: Session,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let user_id: Option<Uuid> = session.get("user_id").await.unwrap_or(None);

    if let Some(uid) = user_id {
        let auth_service = AuthService::new(state.db.clone());
        match auth_service.find_user_by_id(uid).await {
            Ok(Some(user)) => Ok(Json(json!({
                "status": "authenticated",
                "user": {
                    "id": user.id,
                    "username": user.username,
                    "email": user.email,
                    "github_id": user.github_id
                }
            }))),
            _ => Ok(Json(json!({ "status": "unauthenticated" }))),
        }
    } else {
        Ok(Json(json!({ "status": "unauthenticated" })))
    }
}

pub async fn logout_handler(session: Session) -> Json<Value> {
    session.clear().await;
    Json(json!({ "status": "success", "message": "Logged out successfully" }))
}
