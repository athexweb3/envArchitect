use axum::{http, routing::get, Json, Router};
use database::Database;
use dotenv::dotenv;
use serde_json::json;
use std::net::SocketAddr;
use std::sync::Arc;

mod handlers;
mod middleware;
mod services;
mod state;

use state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load Config
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

    // Connect to Database
    let db = Database::connect(&database_url).await?;
    db.migrate().await?;

    // Initialize Redis
    let redis_manager = bb8_redis::RedisConnectionManager::new(redis_url.as_str())?;
    let redis_pool = bb8::Pool::builder().build(redis_manager).await?;

    // Initialize Search Engine
    let search_engine = Arc::new(services::search::engine::SearchEngine::new()?);

    // Initialize TUF Service
    let signing_key_pem = std::env::var("TUF_SIGNING_KEY").unwrap_or_else(|_| {
        tracing::warn!("TUF_SIGNING_KEY not set, using placeholder");
        "placeholder_key".to_string()
    });
    let tuf_service = Arc::new(services::tuf::TufService::new(
        db.pool.clone(),
        &signing_key_pem,
    ));

    // Initialize Worker Channel (stub for now - actual worker not started here)
    let (worker_tx, _worker_rx) = tokio::sync::mpsc::channel::<Box<dyn Send + Sync + 'static>>(100);

    // Create AppState
    let app_state = AppState {
        db: db.clone(),
        worker_tx,
        search_engine,
        tuf_service,
        redis: redis_pool,
    };

    // Setup CORS
    let cors = tower_http::cors::CorsLayer::new()
        .allow_origin(
            "http://localhost:3001"
                .parse::<http::HeaderValue>()
                .unwrap(),
        )
        .allow_methods([
            http::Method::GET,
            http::Method::POST,
            http::Method::PUT,
            http::Method::DELETE,
        ])
        .allow_headers([
            http::header::CONTENT_TYPE,
            http::header::AUTHORIZATION,
            http::header::ACCEPT,
        ])
        .allow_credentials(true);

    // Initialize Session Store
    let session_store = tower_sessions::MemoryStore::default();
    let session_layer = tower_sessions::SessionManagerLayer::new(session_store)
        .with_secure(false) // Set to true in production with HTTPS
        .with_same_site(tower_sessions::cookie::SameSite::Lax);

    // Setup Router using handlers
    let app = Router::new()
        .route("/health", get(health_check))
        .merge(handlers::auth::router())
        .merge(handlers::registry::router())
        .merge(handlers::portal::router())
        .merge(handlers::tuf::router())
        .layer(axum::middleware::from_fn_with_state(
            app_state.clone(),
            middleware::auth::auth_middleware,
        ))
        .layer(axum::extract::DefaultBodyLimit::max(50 * 1024 * 1024))
        .layer(session_layer)
        .layer(cors)
        .with_state(app_state);

    // Start Server
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("Registry API listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> Json<serde_json::Value> {
    Json(json!({ "status": "ok" }))
}
