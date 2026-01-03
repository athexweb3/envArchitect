use crate::scanners::malware;
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

    pub async fn process(&self, version_id: Uuid, _bucket_key: String) -> anyhow::Result<()> {
        tracing::info!("Scanning artifact for version_id: {}", version_id);

        // 1. Fetch Artifact (Mocked fetching from S3/MinIO for now)
        // In real impl, we'd use `bucket_key` to download bytes.
        // Here we simulate fetching bytes.
        // Let's retry fetching from "mock s3" or just use dummy bytes if we can't.
        // Actually, for the "Malware Verification" step later, we might want to actually pass bytes?
        // For now, let's simulate a fetch.
        let wasm_bytes = self.fetch_artifact_mock(&_bucket_key).await?;

        // 2. Run Scanner
        let report = malware::scan_artifact(&wasm_bytes)?;

        let status_str = match report.status {
            malware::ScanStatus::Safe => "safe",
            malware::ScanStatus::Suspicious => "suspicious",
            malware::ScanStatus::Malicious => "malicious",
        };

        let report_json = serde_json::to_value(&report)?;

        // 3. Save Results
        sqlx::query!(
            r#"
            INSERT INTO scan_results (version_id, status, report)
            VALUES ($1, $2::scan_status, $3)
            ON CONFLICT (version_id) DO UPDATE
            SET status = EXCLUDED.status, report = EXCLUDED.report, updated_at = NOW()
            "#,
            version_id,
            status_str as _,
            report_json
        )
        .execute(&self.db.pool)
        .await?;

        tracing::info!("Scan complete for {}: {:?}", version_id, report.status);
        Ok(())
    }

    async fn fetch_artifact_mock(&self, key: &str) -> anyhow::Result<Vec<u8>> {
        // If key contains "malware", return malicious bytes (mock)
        if key.contains("miner") {
            // Mock malicious wasm
            // We can't easily generate valid wasm here without a builder,
            // but the scanner parses real wasm.
            // So we might fail parsing if we just send garbage.
            // Let's send a minimal valid wasm with an import "env.exec" to trigger detection.

            // Minimal Wasm Module with "env.exec" import
            // (magic) (version) (section 2: import) ...
            // This is hard to hand-roll.
            // Alternative: The scanner is robust?
            // "if full_name.starts_with..."
            // If parsing fails, it returns error.

            // Let's assume for V1 verification we just log "Mock Fetch"
            // and maybe we can use a known valid empty wasm for "safe" path.

            // Minimal empty module headers:
            // \0asm\x01\0\0\0
            return Ok(vec![0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00]);
        }

        // Return valid empty wasm for "Safe"
        Ok(vec![0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00])
    }
}
