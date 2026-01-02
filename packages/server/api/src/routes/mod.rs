pub mod auth;
pub mod publish;
pub mod search;

use axum::Router;
use database::Database;
use std::sync::Arc;

/// Merges all sub-routers into the main API router
pub fn router() -> Router<Arc<Database>> {
    Router::new()
        .merge(auth::router())
        .merge(publish::router())
        .merge(search::router())
}
