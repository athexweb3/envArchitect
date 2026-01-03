#[derive(Debug, Clone)]
pub enum Job {
    ScanPlugin {
        version_id: uuid::Uuid,
        name: String,
        version: String,
        bucket_key: String,
    },
    Cleanup,
}
