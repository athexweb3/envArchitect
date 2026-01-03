use crate::services::search::engine::SearchEngine;
use database::Database;
use std::sync::Arc;
use worker::jobs::Job;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Database>,
    pub worker_tx: tokio::sync::mpsc::Sender<Job>,
    pub search_engine: Arc<SearchEngine>,
    pub tuf_service: Arc<crate::services::tuf::TufService>,
    pub redis: bb8::Pool<bb8_redis::RedisConnectionManager>,
}
