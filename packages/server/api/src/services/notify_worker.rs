use shared::dto::PublishPayload;
use tokio::sync::mpsc;

pub enum Job {
    Scan(PublishPayload),
}

pub struct WorkerService {
    tx: mpsc::Sender<Job>,
}

impl WorkerService {
    pub fn new(tx: mpsc::Sender<Job>) -> Self {
        Self { tx }
    }

    pub async fn dispatch(&self, job: Job) -> anyhow::Result<()> {
        self.tx.send(job).await.map_err(|e| anyhow::anyhow!(e))
    }
}
