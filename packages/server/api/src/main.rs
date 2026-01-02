use axum::{extract::State, routing::get, Json, Router};
use database::Database;
use dotenv::dotenv;
use serde_json::json;
use std::net::SocketAddr;
use std::sync::Arc;

mod routes;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load Config
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // Connect to Database (Returns Arc<Database>)
    let db = Database::connect(&database_url).await?;

    // Run Migrations
    db.migrate().await?;

    // Setup Router
    let app = Router::new()
        .route("/health", get(health_check))
        .merge(routes::router()) // Mount all API routes
        .with_state(db); // Pass Arc<Database> as state

    // Start Server
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("Registry API listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check(State(db): State<Arc<Database>>) -> Json<serde_json::Value> {
    match db.health_check().await {
        Ok(_) => Json(json!({ "status": "ok", "database": "connected" })),
        Err(e) => Json(json!({ "status": "error", "database": e.to_string() })),
    }
}
