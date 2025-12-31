use axum::{
    extract::{Query, State},
    routing::get,
    Json, Router,
};
use database::Database;
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;

pub fn router() -> Router<Arc<Database>> {
    Router::new().route("/search", get(search_handler))
}

#[derive(Deserialize)]
struct SearchQuery {
    q: Option<String>,
}

async fn search_handler(
    State(_db): State<Arc<Database>>,
    Query(query): Query<SearchQuery>,
) -> Json<Value> {
    // TODO: Perform Full Text Search on Postgres
    Json(json!({
        "results": [],
        "query": query.q
    }))
}
