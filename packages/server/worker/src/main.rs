use database::Database;
use dotenv::dotenv;
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    tracing::info!("Notary Worker starting...");

    // Connect to shared database
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let _db = Database::connect(&database_url).await?;

    tracing::info!("Connected to database. Waiting for jobs...");

    // Channel for Job Queue
    let (tx, rx) = tokio::sync::mpsc::channel(100);

    // Start the Worker (Consumer)
    let notary = notary_worker::Notary::new(_db.clone(), rx);
    tokio::spawn(async move {
        if let Err(e) = notary.run().await {
            tracing::error!("Notary worker crashed: {:?}", e);
        }
    });

    tracing::info!("Worker pipeline started. Polling for pending plugins...");

    // Producer Loop
    loop {
        // Query for plugins that haven't been scanned yet
        let pending_scans = sqlx::query_as::<_, (Uuid, String, String, String)>(
            r#"
            SELECT 
                pv.id as version_id,
                p.name as package_name,
                pv.version_raw as version,
                pv.oci_reference
            FROM package_versions pv
            JOIN packages p ON pv.package_id = p.id
            LEFT JOIN scan_results sr ON pv.id = sr.version_id
            WHERE sr.version_id IS NULL
            AND pv.oci_reference IS NOT NULL
            LIMIT 10
            "#,
        )
        .fetch_all(&_db.pool)
        .await;

        match pending_scans {
            Ok(recs) => {
                if !recs.is_empty() {
                    tracing::info!("Found {} pending scans", recs.len());
                }

                for rec in recs {
                    let job = notary_worker::jobs::Job::ScanPlugin {
                        version_id: rec.0,
                        name: rec.1,
                        version: rec.2,
                        bucket_key: rec.3,
                    };

                    if let Err(e) = tx.send(job).await {
                        tracing::error!("Failed to enqueue job: {:?}", e);
                    }
                }
            }
            Err(e) => {
                tracing::error!("Failed to poll database: {:?}", e);
            }
        }

        sleep(Duration::from_secs(5)).await;
    }
}
