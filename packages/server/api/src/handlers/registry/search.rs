use crate::handlers::registry::ServiceError;
use crate::state::AppState;
use axum::{
    extract::{Query, State},
    Json,
};
use serde::{Deserialize, Serialize};

// Import shared SearchResult from service
use crate::services::search::engine::SearchResult;

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: String,
    pub limit: Option<i64>,
}

#[derive(Serialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
}

pub async fn search_handler(
    State(state): State<AppState>,
    Query(params): Query<SearchQuery>,
) -> Result<Json<SearchResponse>, ServiceError> {
    let limit = params.limit.unwrap_or(20);
    let q = params.q.trim();

    let results = state
        .search_engine
        .search(&state.db.pool, q, limit)
        .await
        .map_err(|e| {
            tracing::error!("Search failed: {}", e);
            ServiceError::DatabaseError("Search execution failed".to_string())
        })?;

    Ok(Json(SearchResponse { results }))
}
