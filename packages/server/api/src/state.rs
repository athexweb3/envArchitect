use crate::services::search::engine::SearchEngine;
use database::Database;
use std::sync::Arc;

// Generic job type to avoid dependency on worker crate
pub type JobSender = tokio::sync::mpsc::Sender<Box<dyn Send + Sync + 'static>>;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Database>,
    #[allow(dead_code)] // Used by handlers via State extraction
    pub worker_tx: JobSender,
    pub search_engine: Arc<SearchEngine>,
    pub tuf_service: Arc<crate::services::tuf::TufService>,
    #[allow(dead_code)] // Used by middleware
    pub redis: bb8::Pool<bb8_redis::RedisConnectionManager>,
}
