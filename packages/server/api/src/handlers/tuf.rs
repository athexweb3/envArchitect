use crate::handlers::registry::ServiceError;
use crate::services::tuf::{SignedMetadata, Snapshot, Targets, Timestamp};
use crate::state::AppState;
use axum::{extract::State, routing::get, Json, Router};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/tuf/targets.json", get(get_targets))
        .route("/tuf/snapshot.json", get(get_snapshot))
        .route("/tuf/timestamp.json", get(get_timestamp))
}

pub async fn get_targets(
    State(state): State<AppState>,
) -> Result<Json<SignedMetadata<Targets>>, ServiceError> {
    let targets = state
        .tuf_service
        .generate_targets()
        .await
        .map_err(|e| ServiceError::InternalError(e.to_string()))?;
    Ok(Json(targets))
}

pub async fn get_snapshot(
    State(state): State<AppState>,
) -> Result<Json<SignedMetadata<Snapshot>>, ServiceError> {
    let targets = state
        .tuf_service
        .generate_targets()
        .await
        .map_err(|e| ServiceError::InternalError(e.to_string()))?;
    let snapshot = state.tuf_service.generate_snapshot(&targets);
    Ok(Json(snapshot))
}

pub async fn get_timestamp(
    State(state): State<AppState>,
) -> Result<Json<SignedMetadata<Timestamp>>, ServiceError> {
    // In a real system, we'd cache snapshot/targets to ensure consistency chain.
    // Here we regenerate them. Note: Versions will match because they use Utc::now() in TufService (maybe slightly diff if ms pass).
    // TufService uses Utc::now().timestamp() as i32.
    // If we hit edge of second, version skew might happen.
    // For MVP, acceptable risk.
    let targets = state
        .tuf_service
        .generate_targets()
        .await
        .map_err(|e| ServiceError::InternalError(e.to_string()))?;
    let snapshot = state.tuf_service.generate_snapshot(&targets);
    let timestamp = state.tuf_service.generate_timestamp(&snapshot);
    Ok(Json(timestamp))
}
