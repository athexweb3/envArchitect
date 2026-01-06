use shared::dto::PublishPayload;
use tokio::sync::mpsc;

// Stub for Worker Job dispatch (worker crate unavailable)
#[allow(dead_code)]
#[derive(Debug)]
pub enum Job {
    Scan(PublishPayload),
}

#[allow(dead_code)]
pub struct WorkerService {
    tx: mpsc::Sender<Job>,
}

impl WorkerService {
    #[allow(dead_code)]
    pub fn new(tx: mpsc::Sender<Job>) -> Self {
        Self { tx }
    }

    #[allow(dead_code)]
    pub async fn dispatch(&self, job: Job) -> anyhow::Result<()> {
        self.tx.send(job).await.map_err(|e| anyhow::anyhow!(e))
    }
}
