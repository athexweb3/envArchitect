use crate::handlers::registry::ServiceError;
use crate::state::AppState;
use axum::{
    extract::{Path, State},
    Json,
};
use serde::Serialize;

#[derive(Serialize, sqlx::FromRow)]
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
    let dependents = sqlx::query_as::<_, DependentResult>(
        r#"
        SELECT DISTINCT ON (c.name)
            c.name,
            c.version,
            COALESCE(p.score_authority, 0.0) as authority_score,
            COALESCE(p.score_trending, 0.0) as trending_score
        FROM dependencies d
        JOIN components t ON d.target_id = t.id
        JOIN components c ON d.source_id = c.id
        LEFT JOIN packages p ON c.name = p.name
        WHERE t.name = $1
        ORDER BY c.name, p.score_authority DESC
        "#,
    )
    .bind(name)
    .fetch_all(&state.db.pool)
    .await
    .map_err(|e: sqlx::Error| ServiceError::DatabaseError(e.to_string()))?;

    let mut dependents = dependents;
    dependents.sort_by(|a, b| b.authority_score.partial_cmp(&a.authority_score).unwrap());

    Ok(Json(DependentsResponse { dependents }))
}
