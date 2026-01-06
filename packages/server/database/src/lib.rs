use anyhow::{Context, Result};
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::sync::Arc;
use std::time::Duration;

pub use sqlx; // Re-export for convenience
pub mod models;
pub mod repositories;

#[derive(Clone)]
pub struct Database {
    pub pool: PgPool,
}

impl Database {
    /// Connects to the PostgreSQL database with production-ready pool settings.
    pub async fn connect(database_url: &str) -> Result<Arc<Self>> {
        let pool = PgPoolOptions::new()
            .max_connections(50) // Reasonable default for standard Postgres config
            .min_connections(5) // Keep some warm connections
            .acquire_timeout(Duration::from_secs(3)) // Fail fast if DB is overloaded
            .idle_timeout(Duration::from_secs(60 * 5)) // Close idle connections after 5m
            .test_before_acquire(true) // Health check on checkout
            .connect(database_url)
            .await
            .context("Failed to connect to the database")?;

        Ok(Arc::new(Self { pool }))
    }

    /// Runs pending migrations. Safe to run on startup due to Postgres advisory locks.
    pub async fn migrate(&self) -> Result<()> {
        // Embed the migration files into the binary during compilation
        sqlx::migrate!("src/migrations")
            .run(&self.pool)
            .await
            .context("Failed to run database migrations")?;

        Ok(())
    }

    /// Health check for the database connection
    pub async fn health_check(&self) -> Result<()> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .context("Database health check failed")?;
        Ok(())
    }
}
