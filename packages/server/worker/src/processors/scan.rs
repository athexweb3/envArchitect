use crate::scanners::malware;
use crate::services::fetcher::{ArtifactFetcher, GhcrFetcher};
use database::Database;
use std::sync::Arc;
use uuid::Uuid;

pub struct ScanProcessor {
    db: Arc<Database>,
}

impl ScanProcessor {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    pub async fn process(&self, version_id: Uuid, oci_reference: String) -> anyhow::Result<()> {
        tracing::info!("Scanning artifact for version_id: {}", version_id);

        let fetcher = GhcrFetcher::new(self.db.clone());
        let wasm_bytes = fetcher.fetch(&oci_reference, version_id).await?;

        let report = malware::scan_artifact(&wasm_bytes)?;

        let status_str = match report.status {
            malware::ScanStatus::Safe => "safe",
            malware::ScanStatus::Suspicious => "suspicious",
            malware::ScanStatus::Malicious => "malicious",
        };

        let report_json = serde_json::to_value(&report)?;

        sqlx::query(
            r#"
            INSERT INTO scan_results (version_id, status, report)
            VALUES ($1, $2::scan_status, $3)
            ON CONFLICT (version_id) DO UPDATE
            SET status = EXCLUDED.status, report = EXCLUDED.report, updated_at = NOW()
            "#,
        )
        .bind(version_id)
        .bind(status_str)
        .bind(report_json)
        .execute(&self.db.pool)
        .await?;

        tracing::info!("Scan complete for {}: {:?}", version_id, report.status);
        Ok(())
    }
}
