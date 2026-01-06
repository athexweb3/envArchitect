use crate::state::AppState;
use axum::{extract::State, Json};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ScanResultResponse {
    pub packge_name: String,
    pub version: String,
    pub status: String,
    pub score: u8,
    pub report: serde_json::Value,
    pub created_at: String,
}

pub async fn get_dashboard_stats(
    State(state): State<AppState>,
) -> Result<Json<Vec<ScanResultResponse>>, String> {
    let db = &state.db;

    // Explicit anonymous record for non-macro query_as
    let recs = sqlx::query_as::<
        _,
        (
            String,
            String,
            Option<String>,
            serde_json::Value,
            chrono::DateTime<chrono::Utc>,
        ),
    >(
        r#"
        SELECT 
            p.name as package_name,
            pv.version_raw as version,
            sr.status::text as status,
            sr.report,
            sr.created_at
        FROM scan_results sr
        JOIN package_versions pv ON sr.version_id = pv.id
        JOIN packages p ON pv.package_id = p.id
        ORDER BY sr.created_at DESC
        LIMIT 50
        "#,
    )
    .fetch_all(&db.pool)
    .await
    .map_err(|e: sqlx::Error| e.to_string())?;

    let response = recs
        .into_iter()
        .map(|r| {
            let report_val: serde_json::Value = r.3;
            let score = report_val
                .get("score")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u8;

            ScanResultResponse {
                packge_name: r.0,
                version: r.1,
                status: r.2.unwrap_or_default(),
                score,
                report: report_val,
                created_at: r.4.to_string(),
            }
        })
        .collect();

    Ok(Json(response))
}
