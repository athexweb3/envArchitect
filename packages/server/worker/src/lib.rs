use database::Database;
use std::sync::Arc;
use tokio::sync::mpsc::Receiver;

pub struct Notary {
    _db: Arc<Database>,
    rx: Receiver<crate::jobs::Job>,
}

pub mod jobs;
pub mod processors;
pub mod scanners;
pub mod services;
pub mod tasks;

impl Notary {
    pub fn new(db: Arc<Database>, rx: Receiver<crate::jobs::Job>) -> Self {
        Self { _db: db, rx }
    }

    pub async fn run(mut self) -> anyhow::Result<()> {
        tracing::info!("Notary Worker starting...");

        while let Some(job) = self.rx.recv().await {
            tracing::info!("Processing job: {:?}", job);
            match job {
                crate::jobs::Job::ScanPlugin {
                    version_id,
                    bucket_key,
                    ..
                } => {
                    let processor = processors::scan::ScanProcessor::new(self._db.clone());
                    if let Err(e) = processor.process(version_id, bucket_key).await {
                        tracing::error!("Failed to process scan: {:?}", e);
                    }
                }
                crate::jobs::Job::Cleanup => {
                    // TODO
                }
            }
        }

        Ok(())
    }
}
