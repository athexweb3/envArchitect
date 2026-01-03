use crate::handlers::registry::ServiceError;
use crate::state::AppState;
use axum::{
    extract::{Path, State},
    Json,
};
use serde::Serialize;
// use uuid::Uuid;

#[derive(Serialize)]
pub struct DependentResult {
    pub name: String,
    pub version: String,
    pub authority_score: f32,
    pub trending_score: f32,
}

#[derive(Serialize)]
pub struct DependentsResponse {
    pub dependents: Vec<DependentResult>,
}

pub async fn list_dependents(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<DependentsResponse>, ServiceError> {
    // 1. Find all components that depend on the target package `name`.
    // We join:
    // - components (t): The target package (to filter by name)
    // - dependencies (d): The link
    // - components (c): The source package (the dependent)
    // - packages (p): To get the Authority Score of the dependent for sorting

    let dependents = sqlx::query_as!(
        DependentResult,
        r#"
        SELECT DISTINCT ON (c.name)
            c.name,
            c.version,
            COALESCE(p.score_authority, 0.0) as "authority_score!",
            COALESCE(p.score_trending, 0.0) as "trending_score!"
        FROM dependencies d
        JOIN components t ON d.target_id = t.id
        JOIN components c ON d.source_id = c.id
        LEFT JOIN packages p ON c.name = p.name
        WHERE t.name = $1
        ORDER BY c.name, p.score_authority DESC
        -- Note: Distinct on name to avoid showing multiple versions of the same dependent.
        -- We pick one (usually latest due to how we insert, or we should strictly order by version).
        "#,
        name
    )
    .fetch_all(&state.db.pool)
    .await
    .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

    // Re-sort in memory or adjust SQL to sort by authority properly after DISTINCT.
    // SQL `DISTINCT ON (c.name) ORDER BY c.name, ...` requires c.name to be first.
    // So we need a subquery or just sort in Rust. Sorting in Rust is fine for 50-100 items.

    let mut dependents = dependents;
    dependents.sort_by(|a, b| b.authority_score.partial_cmp(&a.authority_score).unwrap());

    Ok(Json(DependentsResponse { dependents }))
}
