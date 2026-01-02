use database::Database;
use dotenv::dotenv;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    tracing::info!("Notary Worker starting...");

    // Connect to shared database
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let _db = Database::connect(&database_url).await?;

    tracing::info!("Connected to database. Waiting for jobs...");

    // Infinite Loop: Process Pending Plugins
    loop {
        // TODO: Query for PENDING plugins
        // TODO: Validate Sigstore bundle
        // TODO: Sign with Root Key if valid
        // TODO: Update status to APPROVED

        sleep(Duration::from_secs(10)).await;
    }
}
