use crate::handlers;
use crate::state::AppState;
use axum::Router;

pub fn routes() -> Router<AppState> {
    Router::new()
        .merge(handlers::auth::router())
        .merge(handlers::registry::router())
        .route(
            "/v1/tuf/targets.json",
            axum::routing::get(handlers::tuf::get_targets),
        )
        .route(
            "/v1/tuf/snapshot.json",
            axum::routing::get(handlers::tuf::get_snapshot),
        )
        .route(
            "/v1/tuf/timestamp.json",
            axum::routing::get(handlers::tuf::get_timestamp),
        )
}
